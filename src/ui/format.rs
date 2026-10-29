/// Formato legible de bytes para columnas de tabla.
pub fn bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let b = bytes as f64;
    if bytes == 0 {
        "—".to_string()
    } else if b >= GIB {
        format!("{:.1}GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.0}MiB", b / MIB)
    } else {
        format!("{bytes}B")
    }
}

/// Formato de uptime legible (`Nd HH:MM`), para la ficha del host.
pub fn uptime(secs: u64) -> String {
    let days = secs / 86_400;
    let hours = (secs % 86_400) / 3_600;
    let mins = (secs % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours:02}:{mins:02}")
    } else {
        format!("{hours:02}:{mins:02}")
    }
}
