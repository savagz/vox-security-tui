use std::time::Duration;

use tokio::sync::{mpsc, watch};

use crate::config::Config;
use crate::event::AppEvent;
use crate::tools::{self, Env};

/// Re-detecta qué herramientas están instaladas cada `poll.refresh_secs`.
/// Guardar en el overlay de configuración empuja el intervalo nuevo por
/// `config_rx`: recarga en caliente, sin reiniciar el proceso.
pub fn spawn(env: Env, config_rx: watch::Receiver<Config>, tx: mpsc::Sender<AppEvent>) {
    tokio::spawn(async move {
        let mut config_rx = config_rx;
        loop {
            refresh(env, &tx).await;

            let interval_secs = config_rx.borrow().poll.refresh_secs.max(1);
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(interval_secs)) => {}
                _ = config_rx.changed() => {}
            }
        }
    });
}

/// `pub(crate)`: `main.rs` la usa para refrescar al toque después de instalar
/// una herramienta, sin esperar al próximo tick.
pub(crate) async fn refresh(env: Env, tx: &mpsc::Sender<AppEvent>) {
    let states = tools::detect_states(env);
    let _ = tx.send(AppEvent::ToolsUpdated(states)).await;
}
