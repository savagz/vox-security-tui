use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    NextTab,
    PrevTab,
    /// Salto directo a una sección del menú (`1`..`5`).
    JumpTab(usize),
    Up,
    Down,
    Top,
    Bottom,
    OpenConfig,
    /// Fuerza re-detección de herramientas (y re-verificación en SYSTEM CHECK).
    Refresh,
    Help,
    /// Filtro por nombre/categoría en el Catálogo.
    Filter,
    /// Abre el popup de detalle de la herramienta seleccionada (Catálogo).
    Info,
    /// Enter: en Catálogo selecciona la herramienta para ejecutar; en Logs abre
    /// la corrida seleccionada.
    Enter,
    /// Abre el formulario de corrida (pestaña EJECUTAR).
    Run,
    /// Repite la última corrida (pestaña EJECUTAR).
    Rerun,
    /// Instala la herramienta seleccionada con el gestor de paquetes (Catálogo).
    Install,
    /// Ejecuta `<binario> --help` de la herramienta seleccionada (si está
    /// instalada) y muestra la salida en EJECUTAR.
    ToolHelp,
    /// Instala la herramienta seleccionada por vía alternativa (go install o
    /// COPR) cuando no está en los repos oficiales (Catálogo).
    InstallAlt,
    /// Limpia la salida de la corrida actual (pestaña EJECUTAR).
    ClearOutput,
    /// Esc: cierra/vuelve.
    Back,
}

pub fn action_for(key: KeyEvent) -> Option<AppAction> {
    match key.code {
        KeyCode::Char('q') => Some(AppAction::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(AppAction::Quit),
        KeyCode::Char('c') => Some(AppAction::OpenConfig),
        KeyCode::F(1) => Some(AppAction::OpenConfig),
        KeyCode::Tab => Some(AppAction::NextTab),
        KeyCode::BackTab => Some(AppAction::PrevTab),
        KeyCode::Char(d @ '1'..='5') => {
            Some(AppAction::JumpTab(d.to_digit(10).unwrap() as usize - 1))
        }
        KeyCode::Up | KeyCode::Char('k') => Some(AppAction::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(AppAction::Down),
        KeyCode::Home | KeyCode::Char('g') => Some(AppAction::Top),
        KeyCode::End | KeyCode::Char('G') => Some(AppAction::Bottom),
        KeyCode::Char('r') => Some(AppAction::Refresh),
        KeyCode::Char('R') => Some(AppAction::Rerun),
        KeyCode::Char('h') | KeyCode::Char('H') => Some(AppAction::Help),
        KeyCode::Char('/') => Some(AppAction::Filter),
        KeyCode::Char('i') => Some(AppAction::Info),
        KeyCode::Char('n') | KeyCode::Char('N') => Some(AppAction::Run),
        KeyCode::Char('I') => Some(AppAction::Install),
        KeyCode::Char('A') => Some(AppAction::InstallAlt),
        KeyCode::Char('?') => Some(AppAction::ToolHelp),
        KeyCode::Char('x') | KeyCode::Char('X') => Some(AppAction::ClearOutput),
        KeyCode::Enter => Some(AppAction::Enter),
        KeyCode::Esc => Some(AppAction::Back),
        _ => None,
    }
}
