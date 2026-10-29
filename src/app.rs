use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use tokio::sync::watch;

use crate::config::{Config, ConfigForm, ConfigOutcome};
use crate::event::AppEvent;
use crate::i18n::{self, Key, Lang};
use crate::keys::{self, AppAction};
use crate::setup::SystemCheckState;
use crate::sysmon::snapshot::HostSnapshot;
use crate::theme::{Status, Theme};
use crate::tools::{self, Env, Tool, ToolState};
use crate::ui::tabs::Tab;

/// Máximo de líneas de salida que se guardan por corrida: evita consumo de
/// memoria sin tope en herramientas muy verbosas.
const MAX_OUTPUT_LINES: usize = 5000;

/// Canal de recarga en caliente hacia el poller. Guardar en el overlay de
/// configuración empuja la config nueva por acá.
pub struct Reload {
    pub config: watch::Sender<Config>,
}

/// Acciones que se resuelven fuera de `App`: necesitan async (ejecutar la
/// herramienta) o adueñarse de la terminal (instalar con `sudo`).
#[derive(Debug, Clone)]
pub enum PendingAction {
    RunScan { run_id: u64, program: String, args: Vec<String> },
    Install(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Finished { code: i32, ok: bool },
}

/// Una corrida registrada (viva o terminada). Alimenta tanto la pestaña
/// EJECUTAR (la más reciente) como el Historial.
#[derive(Debug, Clone)]
pub struct RunRecord {
    pub id: u64,
    pub tool_name: String,
    pub command: String,
    pub started: String,
    pub status: RunStatus,
    pub output: Vec<String>,
}

/// Formulario de corrida (tecla `n` en EJECUTAR): objetivo + argumentos.
#[derive(Debug, Clone)]
pub struct RunForm {
    pub tool_id: &'static str,
    pub focus: usize,
    /// Índice de la plantilla elegida (dentro de `tool.templates`).
    pub template_idx: usize,
    pub target: String,
    pub args: String,
    pub error: Option<String>,
}

impl RunForm {
    pub const FIELDS: usize = 2;
}

pub struct App {
    pub config: Config,
    pub reload: Reload,
    pub env: Env,
    pub lang: Lang,
    pub theme: Theme,
    pub active_tab: Tab,
    pub should_quit: bool,
    /// Contador de ticks para animaciones (spinner de actividad).
    pub tick: u64,

    pub system_check: SystemCheckState,
    pub host: HostSnapshot,
    pub tool_states: Vec<ToolState>,

    // Catálogo
    pub catalog_row: usize,
    pub filter_active: bool,
    pub filter_input: String,

    // Ejecución
    pub selected_tool: Option<&'static str>,
    pub runs: Vec<RunRecord>,
    pub scan_scroll: u16,
    next_run_id: u64,
    /// Última corrida (tool_id, objetivo, plantilla de args) para repetir con `R`.
    last_run: Option<(&'static str, String, String)>,

    // Historial
    pub logs_row: usize,

    // Overlays
    pub config_form: Option<ConfigForm>,
    pub form: Option<RunForm>,
    pub help_open: bool,
    pub info_open: bool,
    pub info_tool: Option<&'static str>,
    pub info_scroll: u16,

    pub toast: Option<(String, Status)>,
    pending: Option<PendingAction>,
}

impl App {
    pub fn new(config: Config, reload: Reload, env: Env, system_check: SystemCheckState) -> Self {
        let lang = config.ui.language;
        Self {
            lang,
            theme: Theme::default(),
            active_tab: Tab::Overview,
            should_quit: false,
            tick: 0,
            system_check,
            host: HostSnapshot::default(),
            tool_states: tools::detect_states(env),
            catalog_row: 0,
            filter_active: false,
            filter_input: String::new(),
            selected_tool: None,
            runs: Vec::new(),
            scan_scroll: 0,
            next_run_id: 1,
            last_run: None,
            logs_row: 0,
            config_form: None,
            form: None,
            help_open: false,
            info_open: false,
            info_tool: None,
            info_scroll: 0,
            toast: None,
            pending: None,
            config,
            reload,
            env,
        }
    }

    pub fn take_pending(&mut self) -> Option<PendingAction> {
        self.pending.take()
    }

    pub fn on_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => self.tick = self.tick.wrapping_add(1),
            AppEvent::Key(key) => {
                if key.kind != KeyEventKind::Release {
                    self.toast = None;
                    self.on_key(key);
                }
            }
            AppEvent::ToolsUpdated(states) => self.tool_states = states,
            AppEvent::ScanLine { run_id, line } => {
                if let Some(rec) = self.runs.iter_mut().find(|r| r.id == run_id) {
                    rec.output.push(line);
                    if rec.output.len() > MAX_OUTPUT_LINES {
                        let overflow = rec.output.len() - MAX_OUTPUT_LINES;
                        rec.output.drain(0..overflow);
                    }
                }
            }
            AppEvent::ScanFinished { run_id, code, ok } => {
                if let Some(rec) = self.runs.iter_mut().find(|r| r.id == run_id) {
                    rec.status = RunStatus::Finished { code, ok };
                }
            }
            AppEvent::SysSnapshot(snapshot) => self.host = snapshot,
            AppEvent::Shutdown => self.should_quit = true,
        }
    }

    // -- helpers de estado --

    pub fn tool_installed(&self, id: &str) -> bool {
        self.tool_states.iter().any(|s| s.id == id && s.installed)
    }

    pub fn installed_count(&self) -> usize {
        self.tool_states.iter().filter(|s| s.installed).count()
    }

    pub fn filtered_tools(&self) -> Vec<&'static Tool> {
        let needle = self.filter_input.to_ascii_lowercase();
        tools::catalog::ALL
            .iter()
            .filter(|t| {
                if needle.is_empty() {
                    return true;
                }
                t.name.to_ascii_lowercase().contains(&needle)
                    || t.category.label(self.lang).to_ascii_lowercase().contains(&needle)
                    || t.id.contains(&needle)
            })
            .collect()
    }

    pub fn selected_catalog_tool(&self) -> Option<&'static Tool> {
        self.filtered_tools().get(self.catalog_row).copied()
    }

    /// Corrida que muestra la pestaña EJECUTAR: la más reciente de la
    /// herramienta seleccionada, o la última en general.
    pub fn current_run(&self) -> Option<&RunRecord> {
        self.runs.first()
    }

    pub fn selected_run(&self) -> Option<&RunRecord> {
        self.runs.get(self.logs_row)
    }

    // -- entrada de teclado --

    fn on_key(&mut self, key: KeyEvent) {
        if self.config_form.is_some() {
            self.on_config_key(key);
            return;
        }
        if self.form.is_some() {
            self.on_form_key(key);
            return;
        }
        if self.help_open {
            self.help_open = false;
            return;
        }
        if self.info_open {
            self.on_info_key(key);
            return;
        }
        if self.filter_active {
            self.on_filter_key(key);
            return;
        }

        let Some(action) = keys::action_for(key) else {
            return;
        };
        self.dispatch(action);
    }

    fn dispatch(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::NextTab => self.active_tab = self.active_tab.next(),
            AppAction::PrevTab => self.active_tab = self.active_tab.prev(),
            AppAction::JumpTab(i) => {
                if let Some(tab) = Tab::from_index(i) {
                    self.active_tab = tab;
                }
            }
            AppAction::OpenConfig => self.config_form = Some(ConfigForm::new(&self.config)),
            AppAction::Help => self.help_open = true,
            AppAction::Refresh => self.on_refresh(),
            AppAction::Up => self.move_up(),
            AppAction::Down => self.move_down(),
            AppAction::Top => self.move_top(),
            AppAction::Bottom => self.move_bottom(),
            AppAction::Filter if self.active_tab == Tab::Catalog => {
                self.filter_active = true;
            }
            AppAction::Info if self.active_tab == Tab::Catalog => self.open_info(),
            AppAction::Install if self.active_tab == Tab::Catalog => self.install_selected(),
            AppAction::InstallAlt if self.active_tab == Tab::Catalog => self.install_selected_alt(),
            AppAction::ToolHelp => self.run_tool_help(),
            AppAction::Enter => self.on_enter(),
            AppAction::Run if self.active_tab == Tab::Scan => self.open_run_form(),
            AppAction::Rerun if self.active_tab == Tab::Scan => self.rerun(),
            AppAction::ClearOutput if self.active_tab == Tab::Scan => self.clear_output(),
            AppAction::Back => {}
            _ => {}
        }
    }

    fn on_refresh(&mut self) {
        if self.active_tab == Tab::SystemCheck {
            self.system_check.recheck();
        }
        // La re-detección real la hace el poller; acá sólo forzamos un probe
        // local inmediato para que la UI no espere al próximo tick.
        self.tool_states = tools::detect_states(self.env);
    }

    fn move_up(&mut self) {
        match self.active_tab {
            Tab::Catalog => self.catalog_row = self.catalog_row.saturating_sub(1),
            Tab::Logs => self.logs_row = self.logs_row.saturating_sub(1),
            Tab::Scan => self.scan_scroll = self.scan_scroll.saturating_sub(1),
            Tab::SystemCheck => {
                self.system_check.selected = self.system_check.selected.saturating_sub(1);
            }
            Tab::Overview => {}
        }
    }

    fn move_down(&mut self) {
        match self.active_tab {
            Tab::Catalog => {
                let max = self.filtered_tools().len().saturating_sub(1);
                self.catalog_row = (self.catalog_row + 1).min(max);
            }
            Tab::Logs => {
                let max = self.runs.len().saturating_sub(1);
                self.logs_row = (self.logs_row + 1).min(max);
            }
            Tab::Scan => self.scan_scroll = self.scan_scroll.saturating_add(1),
            Tab::SystemCheck => {
                let max = self.system_check.report.checks.len().saturating_sub(1);
                self.system_check.selected = (self.system_check.selected + 1).min(max);
            }
            Tab::Overview => {}
        }
    }

    fn move_top(&mut self) {
        match self.active_tab {
            Tab::Catalog => self.catalog_row = 0,
            Tab::Logs => self.logs_row = 0,
            Tab::Scan => self.scan_scroll = 0,
            Tab::SystemCheck => self.system_check.selected = 0,
            Tab::Overview => {}
        }
    }

    fn move_bottom(&mut self) {
        match self.active_tab {
            Tab::Catalog => self.catalog_row = self.filtered_tools().len().saturating_sub(1),
            Tab::Logs => self.logs_row = self.runs.len().saturating_sub(1),
            Tab::Scan => self.scan_scroll = u16::MAX,
            Tab::SystemCheck => {
                self.system_check.selected = self.system_check.report.checks.len().saturating_sub(1);
            }
            Tab::Overview => {}
        }
    }

    fn on_enter(&mut self) {
        match self.active_tab {
            Tab::Catalog => self.select_for_scan(),
            // Enter en el historial salta a EJECUTAR mostrando esa corrida.
            Tab::Logs if self.logs_row < self.runs.len() => {
                self.runs.swap(0, self.logs_row);
                self.logs_row = 0;
                self.scan_scroll = 0;
                self.active_tab = Tab::Scan;
            }
            _ => {}
        }
    }

    fn select_for_scan(&mut self) {
        if let Some(tool) = self.selected_catalog_tool() {
            self.selected_tool = Some(tool.id);
            self.active_tab = Tab::Scan;
            self.toast = Some((
                i18n::t(self.lang, Key::MainSentToScanTemplate).replacen("{}", tool.name, 1),
                Status::Info,
            ));
        }
    }

    fn open_info(&mut self) {
        if let Some(tool) = self.selected_catalog_tool() {
            self.info_tool = Some(tool.id);
            self.info_scroll = 0;
            self.info_open = true;
        }
    }

    fn on_info_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.info_scroll = self.info_scroll.saturating_sub(1),
            KeyCode::Down => self.info_scroll = self.info_scroll.saturating_add(1),
            _ => {
                self.info_open = false;
                self.info_tool = None;
            }
        }
    }

    fn install_selected(&mut self) {
        let Some(tool) = self.selected_catalog_tool() else {
            return;
        };
        match tools::install_command(tool, tools::detect_package_manager()) {
            Some(cmd) => self.pending = Some(PendingAction::Install(cmd)),
            None => {
                self.toast = Some((
                    i18n::t(self.lang, Key::MainNoInstallCommand).to_string(),
                    Status::Warning,
                ));
            }
        }
    }

    /// Lanza `<binario> --help` de la herramienta en foco, sólo si está
    /// instalada. Disponible en Catálogo (herramienta seleccionada en la tabla)
    /// y en Ejecutar (herramienta elegida).
    fn run_tool_help(&mut self) {
        let tool = match self.active_tab {
            Tab::Catalog => self.selected_catalog_tool(),
            Tab::Scan => self.selected_tool.and_then(tools::catalog::by_id),
            _ => return,
        };
        let Some(tool) = tool else {
            return;
        };
        if !self.tool_installed(tool.id) {
            self.toast = Some((
                i18n::t(self.lang, Key::ScanNotInstalledTemplate).replacen("{}", tool.name, 1),
                Status::Warning,
            ));
            return;
        }
        self.start_run(tool, String::new(), "--help".to_string());
    }

    /// Instala la herramienta seleccionada por vía alternativa (go install o
    /// COPR), para las que no están en los repos oficiales.
    fn install_selected_alt(&mut self) {
        let Some(tool) = self.selected_catalog_tool() else {
            return;
        };
        match tools::alt_install_command(tool, tools::detect_package_manager()) {
            Some((kind, cmd)) => {
                self.toast = Some((
                    i18n::t(self.lang, Key::MainAltInstallTemplate)
                        .replacen("{}", tool.name, 1)
                        .replacen("{}", kind.label(), 1),
                    Status::Info,
                ));
                self.pending = Some(PendingAction::Install(cmd));
            }
            None => {
                self.toast = Some((
                    i18n::t(self.lang, Key::MainNoAltInstall).to_string(),
                    Status::Warning,
                ));
            }
        }
    }

    fn open_run_form(&mut self) {
        let Some(id) = self.selected_tool else {
            self.toast = Some((
                i18n::t(self.lang, Key::ScanNoToolSelected).to_string(),
                Status::Warning,
            ));
            return;
        };
        let Some(tool) = tools::catalog::by_id(id) else {
            return;
        };
        self.form = Some(RunForm {
            tool_id: tool.id,
            focus: 0,
            template_idx: 0,
            target: String::new(),
            args: tool.default_args().to_string(),
            error: None,
        });
    }

    fn rerun(&mut self) {
        if let Some((id, target, args)) = self.last_run.clone()
            && let Some(tool) = tools::catalog::by_id(id)
        {
            self.start_run(tool, target, args);
        }
    }

    fn clear_output(&mut self) {
        if let Some(rec) = self.runs.first_mut() {
            rec.output.clear();
            self.scan_scroll = 0;
        }
    }

    fn start_run(&mut self, tool: &'static Tool, target: String, args_template: String) {
        let run_id = self.next_run_id;
        self.next_run_id += 1;

        let args = tools::build_args(&args_template, &target);
        let program = tool.binary.to_string();
        let command = if args.is_empty() {
            program.clone()
        } else {
            format!("{program} {}", args.join(" "))
        };

        self.runs.insert(
            0,
            RunRecord {
                id: run_id,
                tool_name: tool.name.to_string(),
                command,
                started: Local::now().format("%H:%M:%S").to_string(),
                status: RunStatus::Running,
                output: Vec::new(),
            },
        );
        self.logs_row = 0;
        self.scan_scroll = 0;
        self.selected_tool = Some(tool.id);
        self.last_run = Some((tool.id, target, args_template));
        self.active_tab = Tab::Scan;
        self.toast = Some((
            i18n::t(self.lang, Key::MainScanStartedTemplate).replacen("{}", tool.name, 1),
            Status::Info,
        ));
        self.pending = Some(PendingAction::RunScan { run_id, program, args });
    }

    fn on_filter_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.filter_active = false;
                self.filter_input.clear();
                self.catalog_row = 0;
            }
            KeyCode::Enter => self.filter_active = false,
            KeyCode::Backspace => {
                self.filter_input.pop();
                self.catalog_row = 0;
            }
            KeyCode::Char(c) => {
                self.filter_input.push(c);
                self.catalog_row = 0;
            }
            _ => {}
        }
    }

    fn on_config_key(&mut self, key: KeyEvent) {
        let Some(form) = self.config_form.as_mut() else {
            return;
        };
        match form.on_key(key) {
            ConfigOutcome::None => {}
            ConfigOutcome::Cancel => self.config_form = None,
            ConfigOutcome::Save(config) => {
                let config = *config;
                self.apply_config(config);
                self.config_form = None;
            }
        }
    }

    fn apply_config(&mut self, config: Config) {
        self.lang = config.ui.language;
        self.system_check.set_lang(self.lang);
        let _ = config.save();
        let _ = self.reload.config.send(config.clone());
        self.config = config;
    }

    fn on_form_key(&mut self, key: KeyEvent) {
        let Some(form) = self.form.as_mut() else {
            return;
        };
        form.error = None;
        match key.code {
            KeyCode::Esc => self.form = None,
            KeyCode::Tab | KeyCode::Down => {
                form.focus = (form.focus + 1) % RunForm::FIELDS;
            }
            KeyCode::Up => {
                form.focus = (form.focus + RunForm::FIELDS - 1) % RunForm::FIELDS;
            }
            KeyCode::Left | KeyCode::Right => {
                if let Some(tool) = tools::catalog::by_id(form.tool_id) {
                    let n = tool.templates.len();
                    if n > 0 {
                        form.template_idx = if key.code == KeyCode::Left {
                            (form.template_idx + n - 1) % n
                        } else {
                            (form.template_idx + 1) % n
                        };
                        form.args = tool.templates[form.template_idx].args.to_string();
                    }
                }
            }
            KeyCode::Backspace => {
                let field = if form.focus == 0 { &mut form.target } else { &mut form.args };
                field.pop();
            }
            KeyCode::Char(c) => {
                let field = if form.focus == 0 { &mut form.target } else { &mut form.args };
                field.push(c);
            }
            KeyCode::Enter => {
                let tool_id = form.tool_id;
                let target = form.target.trim().to_string();
                let args = form.args.clone();
                if args.trim().is_empty() && target.is_empty() {
                    form.error = Some(i18n::t(self.lang, Key::FormErrNoCommand).to_string());
                    return;
                }
                if let Some(tool) = tools::catalog::by_id(tool_id) {
                    self.form = None;
                    self.start_run(tool, target, args);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyModifiers};

    fn app() -> App {
        let config = Config::default();
        let (config_tx, _rx) = watch::channel(config.clone());
        let env = Env { fixtures: true };
        let system_check = SystemCheckState::new(Lang::default(), env);
        App::new(config, Reload { config: config_tx }, env, system_check)
    }

    fn press(app: &mut App, code: KeyCode) {
        app.on_event(AppEvent::Key(KeyEvent::new(code, KeyModifiers::NONE)));
    }

    #[test]
    fn seleccionar_en_catalogo_arma_la_corrida() {
        let mut app = app();
        app.active_tab = Tab::Catalog;
        press(&mut app, KeyCode::Enter);
        assert!(app.selected_tool.is_some());
        assert_eq!(app.active_tab, Tab::Scan);

        press(&mut app, KeyCode::Char('n'));
        assert!(app.form.is_some());
        // El form viene prellenado con los args por defecto de la herramienta.
        let form = app.form.as_ref().unwrap();
        let tool = tools::catalog::by_id(form.tool_id).unwrap();
        assert_eq!(form.args, tool.default_args());

        // Tipeamos un objetivo y lanzamos.
        for c in "ejemplo.local".chars() {
            press(&mut app, KeyCode::Char(c));
        }
        press(&mut app, KeyCode::Enter);
        assert!(app.form.is_none());
        assert_eq!(app.runs.len(), 1);
        assert_eq!(app.runs[0].status, RunStatus::Running);
        match app.take_pending() {
            Some(PendingAction::RunScan { program, .. }) => {
                assert_eq!(program, tool.binary);
            }
            other => panic!("se esperaba RunScan, vino {other:?}"),
        }
    }

    #[test]
    fn ayuda_de_herramienta_instalada_corre_help() {
        let mut app = app();
        app.active_tab = Tab::Catalog;
        // La primera fila del catálogo es nmap, instalada en fixtures.
        assert!(app.tool_installed("nmap"));
        press(&mut app, KeyCode::Char('?'));
        assert_eq!(app.runs.len(), 1);
        match app.take_pending() {
            Some(PendingAction::RunScan { program, args, .. }) => {
                assert_eq!(program, "nmap");
                assert_eq!(args, vec!["--help".to_string()]);
            }
            other => panic!("se esperaba RunScan, vino {other:?}"),
        }
    }

    #[test]
    fn ayuda_de_herramienta_no_instalada_avisa() {
        let mut app = app();
        app.active_tab = Tab::Catalog;
        // masscan no está en el set de fixtures instaladas.
        let pos = app.filtered_tools().iter().position(|t| t.id == "masscan").unwrap();
        app.catalog_row = pos;
        assert!(!app.tool_installed("masscan"));
        press(&mut app, KeyCode::Char('?'));
        assert!(app.runs.is_empty());
        assert!(app.toast.is_some());
    }

    #[test]
    fn el_selector_cicla_plantillas_y_rellena_args() {
        let mut app = app();
        app.selected_tool = Some("nmap");
        app.active_tab = Tab::Scan;
        press(&mut app, KeyCode::Char('n'));
        let tool = tools::catalog::by_id("nmap").unwrap();

        // Arranca en la primera plantilla (args por defecto).
        assert_eq!(app.form.as_ref().unwrap().template_idx, 0);
        assert_eq!(app.form.as_ref().unwrap().args, tool.templates[0].args);

        // ▸ avanza a la segunda y rellena los args.
        press(&mut app, KeyCode::Right);
        assert_eq!(app.form.as_ref().unwrap().template_idx, 1);
        assert_eq!(app.form.as_ref().unwrap().args, tool.templates[1].args);

        // ◂ desde la primera envuelve a la última.
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Left);
        let last = tool.templates.len() - 1;
        assert_eq!(app.form.as_ref().unwrap().template_idx, last);
        assert_eq!(app.form.as_ref().unwrap().args, tool.templates[last].args);
    }

    #[test]
    fn ejecutar_sin_herramienta_avisa() {
        let mut app = app();
        app.active_tab = Tab::Scan;
        press(&mut app, KeyCode::Char('n'));
        assert!(app.form.is_none());
        assert!(app.toast.is_some());
    }

    #[test]
    fn el_filtro_del_catalogo_acota_la_lista() {
        let mut app = app();
        app.active_tab = Tab::Catalog;
        press(&mut app, KeyCode::Char('/'));
        assert!(app.filter_active);
        for c in "nmap".chars() {
            press(&mut app, KeyCode::Char(c));
        }
        assert_eq!(app.filtered_tools().len(), 1);
    }

    #[test]
    fn las_lineas_de_salida_se_acumulan_en_la_corrida() {
        let mut app = app();
        let tool = tools::catalog::by_id("nmap").unwrap();
        app.start_run(tool, "10.0.0.1".to_string(), tool.default_args().to_string());
        let run_id = app.runs[0].id;
        app.on_event(AppEvent::ScanLine { run_id, line: "linea 1".to_string() });
        app.on_event(AppEvent::ScanFinished { run_id, code: 0, ok: true });
        assert_eq!(app.runs[0].output, vec!["linea 1".to_string()]);
        assert_eq!(app.runs[0].status, RunStatus::Finished { code: 0, ok: true });
    }
}
