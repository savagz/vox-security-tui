use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::event::AppEvent;

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
        match run_real(run_id, &program, &args, timeout_secs, &tx).await {
            Ok((code, ok)) => {
                let _ = tx.send(AppEvent::ScanFinished { run_id, code, ok }).await;
            }
            Err(message) => {
                let _ = tx.send(AppEvent::ScanLine { run_id, line: message }).await;
                let _ = tx
                    .send(AppEvent::ScanFinished { run_id, code: -1, ok: false })
                    .await;
            }
        }
    });
}

async fn run_real(
    run_id: u64,
    program: &str,
    args: &[String],
    timeout_secs: u64,
    tx: &mpsc::Sender<AppEvent>,
) -> Result<(i32, bool), String> {
    let mut child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo ejecutar `{program}`: {e}"))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut out_lines = stdout.map(|s| BufReader::new(s).lines());
    let mut err_lines = stderr.map(|s| BufReader::new(s).lines());

    let pump = async {
        loop {
            let next_out = async {
                match &mut out_lines {
                    Some(l) => l.next_line().await,
                    None => Ok(None),
                }
            };
            let next_err = async {
                match &mut err_lines {
                    Some(l) => l.next_line().await,
                    None => Ok(None),
                }
            };

            tokio::select! {
                res = next_out => match res {
                    Ok(Some(line)) => {
                        let line = crate::tools::strip_ansi(&line);
                        let _ = tx.send(AppEvent::ScanLine { run_id, line }).await;
                    }
                    _ => out_lines = None,
                },
                res = next_err => match res {
                    Ok(Some(line)) => {
                        let line = crate::tools::strip_ansi(&line);
                        let _ = tx.send(AppEvent::ScanLine { run_id, line }).await;
                    }
                    _ => err_lines = None,
                },
            }

            if out_lines.is_none() && err_lines.is_none() {
                break;
            }
        }
        child.wait().await
    };

    match timeout(Duration::from_secs(timeout_secs.max(1)), pump).await {
        Ok(Ok(status)) => {
            let code = status.code().unwrap_or(-1);
            Ok((code, status.success()))
        }
        Ok(Err(e)) => Err(format!("error esperando a `{program}`: {e}")),
        Err(_) => Err(format!("`{program}` superó el timeout y fue abortado")),
    }
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
