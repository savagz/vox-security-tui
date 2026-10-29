use crossterm::event::KeyEvent;

use crate::sysmon::snapshot::HostSnapshot;
use crate::tools::ToolState;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Key(KeyEvent),
    /// Resultado de re-detectar qué herramientas están instaladas.
    ToolsUpdated(Vec<ToolState>),
    /// Una línea de salida de la corrida `run_id` (stdout o stderr unificados).
    ScanLine { run_id: u64, line: String },
    /// La corrida `run_id` terminó con el código `code` (`ok` = código 0).
    ScanFinished { run_id: u64, code: i32, ok: bool },
    SysSnapshot(HostSnapshot),
    /// SIGTERM/SIGINT a nivel de proceso: hay que salir ya para restaurar la terminal.
    Shutdown,
}
