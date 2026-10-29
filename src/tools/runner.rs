use std::process::Stdio;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::event::AppEvent;

/// Tope de bytes por línea de salida. Lo que exceda se descarta hasta el
/// próximo salto de línea: una herramienta que vuelque binario o líneas
/// gigantes no puede hacer crecer el buffer sin límite.
const MAX_LINE_BYTES: usize = 4096;

/// Gracia entre SIGTERM y SIGKILL al abortar por timeout.
const KILL_GRACE: Duration = Duration::from_secs(2);

/// Lanza `program args...` en background y transmite su salida línea a línea por
/// `tx` (`ScanLine`), seguido de `ScanFinished`. stdout y stderr se unifican en
/// orden de llegada. En modo fixtures no toca ningún binario: emite una salida
/// simulada. Un timeout global mata el proceso y lo reporta como error.
pub fn spawn(
    fixtures: bool,
    run_id: u64,
    program: String,
    args: Vec<String>,
    timeout_secs: u64,
    tx: mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        if fixtures {
            simulate(run_id, &program, &args, &tx).await;
            return;
        }
        let started = Instant::now();
        match run_real(run_id, &program, &args, timeout_secs, &tx).await {
            Ok((code, ok)) => {
                tracing::info!(
                    run_id,
                    program,
                    code,
                    ok,
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    "corrida terminada"
                );
                let _ = tx.send(AppEvent::ScanFinished { run_id, code, ok }).await;
            }
            Err(message) => {
                tracing::warn!(
                    run_id,
                    program,
                    elapsed_ms = started.elapsed().as_millis() as u64,
                    error = %message,
                    "corrida fallida"
                );
                let _ = tx.send(AppEvent::ScanLine { run_id, line: message }).await;
                let _ = tx
                    .send(AppEvent::ScanFinished { run_id, code: -1, ok: false })
                    .await;
            }
        }
    });
}

/// Mata el grupo de procesos de la herramienta si se descarta sin desarmar:
/// cubre el cierre de la app (el runtime dropea la tarea a mitad de corrida).
/// `kill_on_drop` sólo alcanzaría al hijo directo, no a sus descendientes.
struct GroupGuard(Option<i32>);

impl GroupGuard {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        if let Some(pgid) = self.0 {
            signal_group(pgid, libc::SIGKILL);
        }
    }
}

fn signal_group(pgid: i32, signal: i32) {
    // SAFETY: `kill` no toma punteros; un pgid negativo apunta al grupo entero.
    unsafe {
        libc::kill(-pgid, signal);
    }
}

async fn run_real(
    run_id: u64,
    program: &str,
    args: &[String],
    timeout_secs: u64,
    tx: &mpsc::Sender<AppEvent>,
) -> Result<(i32, bool), String> {
    // Grupo de procesos propio: permite matar a la herramienta y a todo lo que
    // lance, y la aísla de las señales de la terminal (Ctrl+C lo maneja la TUI).
    let mut child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("no se pudo ejecutar `{program}`: {e}"))?;

    let mut guard = GroupGuard(child.id().map(|pid| pid as i32));

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // `join!` (no `select!`): cada lector conserva su línea parcial entre
    // lecturas, cosa que un `select!` cancelaría.
    let pump = async {
        tokio::join!(forward(stdout, run_id, tx), forward(stderr, run_id, tx));
        child.wait().await
    };
    let result = timeout(Duration::from_secs(timeout_secs.max(1)), pump).await;

    match result {
        Ok(Ok(status)) => {
            guard.disarm();
            let code = status.code().unwrap_or(-1);
            Ok((code, status.success()))
        }
        Ok(Err(e)) => Err(format!("error esperando a `{program}`: {e}")),
        Err(_) => {
            if let Some(pgid) = guard.0 {
                signal_group(pgid, libc::SIGTERM);
                if timeout(KILL_GRACE, child.wait()).await.is_err() {
                    signal_group(pgid, libc::SIGKILL);
                    let _ = child.wait().await;
                }
                // Descendientes que hayan ignorado SIGTERM.
                signal_group(pgid, libc::SIGKILL);
                guard.disarm();
            }
            Err(format!("`{program}` superó el timeout de {timeout_secs}s y fue abortado"))
        }
    }
}

async fn forward<R: AsyncRead + Unpin>(
    stream: Option<R>,
    run_id: u64,
    tx: &mpsc::Sender<AppEvent>,
) {
    let Some(stream) = stream else {
        return;
    };
    let mut reader = BufReader::new(stream);
    while let Ok(Some(line)) = read_line_capped(&mut reader, MAX_LINE_BYTES).await {
        let line = crate::tools::strip_ansi(&line);
        if tx.send(AppEvent::ScanLine { run_id, line }).await.is_err() {
            return;
        }
    }
}

/// Lee una línea guardando como máximo `max` bytes; el resto hasta el salto de
/// línea se descarta y la línea se marca con `[…]`. `Ok(None)` al llegar a EOF.
async fn read_line_capped<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> std::io::Result<Option<String>> {
    let mut buf = Vec::new();
    let mut truncated = false;
    let mut read_any = false;
    loop {
        let chunk = reader.fill_buf().await?;
        if chunk.is_empty() {
            if !read_any {
                return Ok(None);
            }
            break;
        }
        read_any = true;
        let (used, done) = match chunk.iter().position(|&b| b == b'\n') {
            Some(i) => (i + 1, true),
            None => (chunk.len(), false),
        };
        let room = max.saturating_sub(buf.len());
        if used > room {
            buf.extend_from_slice(&chunk[..room]);
            truncated = true;
        } else {
            buf.extend_from_slice(&chunk[..used]);
        }
        reader.consume(used);
        if done {
            break;
        }
    }
    while matches!(buf.last(), Some(b'\n' | b'\r')) {
        buf.pop();
    }
    let mut line = String::from_utf8_lossy(&buf).into_owned();
    if truncated {
        line.push_str(" […]");
    }
    Ok(Some(line))
}

async fn simulate(run_id: u64, program: &str, args: &[String], tx: &mpsc::Sender<AppEvent>) {
    let header = format!("$ {} {}", program, args.join(" "));
    let lines = [
        header,
        "[fixtures] ejecución simulada, no se invocó ningún binario".to_string(),
        "[fixtures] 3 objetivos analizados, 0 hallazgos".to_string(),
        "[fixtures] done".to_string(),
    ];
    for line in lines {
        if tx.send(AppEvent::ScanLine { run_id, line }).await.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(180)).await;
    }
    let _ = tx
        .send(AppEvent::ScanFinished { run_id, code: 0, ok: true })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn lines(input: &[u8], max: usize) -> Vec<String> {
        let mut reader = BufReader::new(input);
        let mut out = Vec::new();
        while let Some(line) = read_line_capped(&mut reader, max).await.unwrap() {
            out.push(line);
        }
        out
    }

    #[tokio::test]
    async fn read_line_capped_separa_y_recorta() {
        assert_eq!(lines(b"uno\r\ndos\ntres", 64).await, vec!["uno", "dos", "tres"]);
        assert_eq!(lines(b"abcdefgh\nok\n", 4).await, vec!["abcd […]", "ok"]);
        assert!(lines(b"", 4).await.is_empty());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn el_timeout_mata_al_proceso_y_sus_hijos() {
        let marker = std::env::temp_dir().join(format!("voxsec-timeout-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        // El hijo de `sh` crea el marcador recién a los 3s: si sobrevive al
        // timeout de 1s, el archivo aparece.
        let script = format!("(sleep 3; touch {}) & sleep 30", marker.display());
        let (tx, mut rx) = mpsc::channel(16);
        let started = Instant::now();
        let res = run_real(1, "sh", &["-c".into(), script], 1, &tx).await;
        assert!(res.is_err());
        assert!(started.elapsed() < Duration::from_secs(5));
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert!(!marker.exists(), "un proceso hijo sobrevivió al timeout");
        drop(tx);
        while rx.recv().await.is_some() {}
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn abortar_la_tarea_mata_al_grupo_de_procesos() {
        let marker = std::env::temp_dir().join(format!("voxsec-abort-{}", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let script = format!("(sleep 2; touch {}) & sleep 30", marker.display());
        let (tx, mut rx) = mpsc::channel(16);
        // Simula el cierre de la app: la tarea se cancela a mitad de corrida.
        let task = tokio::spawn(async move {
            let _ = run_real(1, "sh", &["-c".into(), script], 60, &tx).await;
        });
        tokio::time::sleep(Duration::from_millis(500)).await;
        task.abort();
        let _ = task.await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        assert!(!marker.exists(), "un proceso sobrevivió al cierre");
        while rx.try_recv().is_ok() {}
    }
}
