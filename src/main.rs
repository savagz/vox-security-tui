mod app;
mod config;
mod deps;
mod event;
mod i18n;
mod keys;
mod logging;
mod poller;
mod setup;
mod sysmon;
mod theme;
mod tools;
mod ui;

use std::time::Duration;

use app::{App, PendingAction, Reload};
use color_eyre::Result;
use config::Config;
use crossterm::event::{Event, EventStream};
use event::AppEvent;
use futures_util::StreamExt;
use i18n::{Key, Lang};
use setup::{SystemCheckMode, SystemCheckOutcome, SystemCheckState};
use theme::{Status, Theme};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use tools::Env;

const USAGE: &str = "voxsecurity-tui: TUI de auditoría de seguridad (uso autorizado)

USO:
    voxsecurity-tui [OPCIONES]

OPCIONES:
    -V, --version    Muestra la versión y sale
    -h, --help       Muestra esta ayuda y sale

ENTORNO:
    VOXSECURITY_FIXTURES=1    Fuerza el modo fixtures (datos simulados, sin ejecutar binarios)";

fn handle_cli_args() -> bool {
    match std::env::args().nth(1).as_deref() {
        Some("-V" | "--version") => {
            println!("voxsecurity-tui {}", env!("CARGO_PKG_VERSION"));
            true
        }
        Some("-h" | "--help") => {
            println!("{USAGE}");
            true
        }
        _ => false,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if handle_cli_args() {
        return Ok(());
    }
    color_eyre::install()?;
    let _log_guard = logging::init()?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal).await;
    ratatui::restore();
    result
}

async fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);
    let mut input = Input::new(tx.clone());
    input.start();
    spawn_signal_task(tx.clone());

    let config = Config::load().ok().flatten().unwrap_or_default();
    let env = Env::detect();

    // El wizard de verificación sólo bloquea si hay algo en rojo. Como ningún
    // chequeo es bloqueante (la app es un orquestador y arranca igual), en la
    // práctica se salta; se mantiene por consistencia con la familia VOXPBX.
    let mut system_check = SystemCheckState::new(config.ui.language, env);
    if !system_check.report.all_ok() {
        match run_gate(terminal, &mut rx, &mut system_check).await? {
            GateResult::Continue => {}
            GateResult::Quit => return Ok(()),
        }
    }

    run_app(terminal, config, env, system_check, tx, rx, input).await
}

struct Input {
    tx: mpsc::Sender<AppEvent>,
    handle: Option<JoinHandle<()>>,
}

impl Input {
    fn new(tx: mpsc::Sender<AppEvent>) -> Self {
        Self { tx, handle: None }
    }

    fn start(&mut self) {
        if self.handle.is_none() {
            self.handle = Some(spawn_input_task(self.tx.clone()));
        }
    }

    fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

enum GateResult {
    Continue,
    Quit,
}

async fn run_gate(
    terminal: &mut ratatui::DefaultTerminal,
    rx: &mut mpsc::Receiver<AppEvent>,
    state: &mut SystemCheckState,
) -> Result<GateResult> {
    let theme = Theme::default();
    terminal.draw(|frame| ui::setup::render_gate(frame, frame.area(), state, &theme))?;

    while let Some(event) = rx.recv().await {
        match event {
            AppEvent::Key(key) => match state.on_key(key, SystemCheckMode::Gate) {
                SystemCheckOutcome::None => {}
                SystemCheckOutcome::Recheck => state.recheck(),
                SystemCheckOutcome::Continue => return Ok(GateResult::Continue),
                SystemCheckOutcome::Cancel => return Ok(GateResult::Quit),
            },
            AppEvent::Shutdown => return Ok(GateResult::Quit),
            _ => {}
        }
        terminal.draw(|frame| ui::setup::render_gate(frame, frame.area(), state, &theme))?;
    }

    Ok(GateResult::Quit)
}

async fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    config: Config,
    env: Env,
    system_check: SystemCheckState,
    tx: mpsc::Sender<AppEvent>,
    mut rx: mpsc::Receiver<AppEvent>,
    mut input: Input,
) -> Result<()> {
    let tick_rate = Duration::from_millis(config.ui.tick_rate_ms);
    let (config_tx, config_rx) = watch::channel(config.clone());

    spawn_tick_task(tx.clone(), tick_rate);
    sysmon::spawn(tx.clone());
    poller::spawn(env, config_rx, tx.clone());

    let mut app = App::new(config, Reload { config: config_tx }, env, system_check);

    terminal.draw(|frame| ui::draw(frame, &app))?;

    while let Some(event) = rx.recv().await {
        app.on_event(event);

        if let Some(pending) = app.take_pending() {
            match pending {
                PendingAction::RunScan { run_id, program, args } => {
                    tools::runner::spawn(
                        env.fixtures,
                        run_id,
                        program,
                        args,
                        app.config.ui.command_timeout_secs,
                        tx.clone(),
                    );
                }
                PendingAction::Install(cmd) => {
                    // En fixtures no se toca la terminal ni se lanza un
                    // instalador real: sólo se simula y se refresca.
                    let message = if env.fixtures {
                        i18n::t(app.lang, Key::MainFixturesSimulated).to_string()
                    } else {
                        run_install(terminal, &cmd, &mut input, app.lang)
                    };
                    app.toast = Some((message, Status::Info));
                    // Refresco síncrono (no por el canal): mientras `run_install`
                    // bloqueaba el bucle esperando ENTER, las tareas de tick y
                    // poller pudieron llenar el canal. Un `tx.send().await` acá se
                    // auto-bloquearía (el único consumidor es este bucle), dejando
                    // la UI congelada y en blanco. `detect_states` es barato.
                    app.tool_states = tools::detect_states(env);
                }
            }
        }

        if app.should_quit {
            break;
        }
        terminal.draw(|frame| ui::draw(frame, &app))?;
    }

    Ok(())
}

/// Sale de la pantalla alterna, corre el comando de instalación heredando la
/// terminal (para que `sudo` pueda pedir la contraseña), y vuelve a entrar.
fn run_install(
    terminal: &mut ratatui::DefaultTerminal,
    command: &[String],
    input: &mut Input,
    lang: Lang,
) -> String {
    input.stop();
    ratatui::restore();

    println!("\n>> {}\n", tools::command_line(command));
    let outcome = tools::run_install(command);

    println!("\n{}", i18n::t(lang, Key::MainPressEnterToReturn));
    let mut discard = String::new();
    let _ = std::io::stdin().read_line(&mut discard);

    *terminal = ratatui::init();
    // El comando pudo haber scrolleado la pantalla primaria: forzamos un
    // repintado completo del buffer nuevo, si no el tablero queda en blanco.
    let _ = terminal.clear();
    input.start();

    match outcome {
        Ok(true) => i18n::t(lang, Key::MainFixDone).to_string(),
        Ok(false) => i18n::t(lang, Key::MainFixError).to_string(),
        Err(e) => i18n::t(lang, Key::MainFixExecErrTemplate).replacen("{}", &e.to_string(), 1),
    }
}

fn spawn_input_task(tx: mpsc::Sender<AppEvent>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut stream = EventStream::new();
        while let Some(Ok(evt)) = stream.next().await {
            if let Event::Key(key) = evt
                && tx.send(AppEvent::Key(key)).await.is_err()
            {
                break;
            }
        }
    })
}

fn spawn_tick_task(tx: mpsc::Sender<AppEvent>, tick_rate: Duration) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tick_rate);
        loop {
            interval.tick().await;
            if tx.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });
}

/// SIGTERM no llega como KeyEvent de crossterm: sin este handler el proceso
/// muere antes de restaurar la terminal.
fn spawn_signal_task(tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let mut sigterm =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(s) => s,
                Err(_) => return,
            };
        sigterm.recv().await;
        let _ = tx.send(AppEvent::Shutdown).await;
    });
}
