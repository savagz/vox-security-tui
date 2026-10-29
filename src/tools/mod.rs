pub mod catalog;
pub mod runner;

use std::process::Stdio;

use crate::i18n::{Key, Lang};

/// Categorías del catálogo, según `SPEC.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Recon,
    NetScan,
    Web,
    Passwords,
    Wireless,
    Hardening,
    Forensics,
    Traffic,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Recon,
        Category::NetScan,
        Category::Web,
        Category::Passwords,
        Category::Wireless,
        Category::Hardening,
        Category::Forensics,
        Category::Traffic,
    ];

    pub fn key(&self) -> Key {
        match self {
            Category::Recon => Key::CatRecon,
            Category::NetScan => Key::CatNetScan,
            Category::Web => Key::CatWeb,
            Category::Passwords => Key::CatPasswords,
            Category::Wireless => Key::CatWireless,
            Category::Hardening => Key::CatHardening,
            Category::Forensics => Key::CatForensics,
            Category::Traffic => Key::CatTraffic,
        }
    }

    pub fn label(&self, lang: Lang) -> &'static str {
        crate::i18n::t(lang, self.key())
    }
}

/// Plantilla de uso de una herramienta: un caso concreto con nombre y la línea
/// de argumentos lista para editar. `{target}` se reemplaza por el objetivo.
#[derive(Debug, Clone, Copy)]
pub struct Template {
    /// Nombre del caso de uso (es, en).
    pub name: (&'static str, &'static str),
    /// Argumentos; `{target}` se reemplaza por el objetivo.
    pub args: &'static str,
}

impl Template {
    pub fn name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Es => self.name.0,
            Lang::En => self.name.1,
        }
    }
}

/// Entrada del catálogo. Todos los campos son estáticos: la app no inventa
/// paquetes ni flags, usa exactamente los del catálogo (ver `SPEC.md`).
#[derive(Debug, Clone, Copy)]
pub struct Tool {
    pub id: &'static str,
    pub name: &'static str,
    /// Binario a detectar y ejecutar.
    pub binary: &'static str,
    pub category: Category,
    pub pkg_debian: &'static str,
    pub pkg_arch: &'static str,
    pub pkg_fedora: &'static str,
    pub license: &'static str,
    /// Plantillas de uso. La primera es el caso básico y da los argumentos por
    /// defecto del formulario de corrida. Nunca está vacía.
    pub templates: &'static [Template],
    /// Ruta de módulo para `go install <path>` cuando la herramienta no está en
    /// los repos oficiales pero sí disponible como binario Go. `None` si no
    /// aplica.
    pub go_install: Option<&'static str>,
    /// Repo COPR (`usuario/proyecto`) para instalar en Fedora cuando no está en
    /// los repos oficiales. `None` si no hay uno configurado.
    pub copr: Option<&'static str>,
    /// Propósito (es, en).
    pub purpose: (&'static str, &'static str),
}

impl Tool {
    pub fn purpose(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Es => self.purpose.0,
            Lang::En => self.purpose.1,
        }
    }

    /// Argumentos por defecto: los de la primera plantilla (caso básico).
    pub fn default_args(&self) -> &'static str {
        self.templates.first().map(|t| t.args).unwrap_or("")
    }
}

/// Estado de detección de una herramienta (resultado del poller).
#[derive(Debug, Clone)]
pub struct ToolState {
    pub id: &'static str,
    pub installed: bool,
    pub path: Option<String>,
}

/// Entorno de ejecución. `fixtures = true` no toca binarios reales: todo se
/// simula (macOS sin herramientas, o `VOXSECURITY_FIXTURES=1`).
#[derive(Debug, Clone, Copy)]
pub struct Env {
    pub fixtures: bool,
}

impl Env {
    pub fn detect() -> Self {
        let forced = std::env::var("VOXSECURITY_FIXTURES")
            .map(|v| v == "1")
            .unwrap_or(false);
        let any_installed = catalog::ALL.iter().any(|t| which_path(t.binary).is_some());
        let fixtures = forced || (cfg!(target_os = "macos") && !any_installed);
        Env { fixtures }
    }
}

/// Subconjunto que el modo fixtures reporta como "instalado", para que la UI
/// muestre una mezcla realista de estados sin depender del host.
const FIXTURE_INSTALLED: &[&str] = &[
    "nmap", "whois", "nikto", "john", "tcpdump", "binwalk", "exiftool", "lynis", "gobuster",
    "hashid", "whatweb", "arp-scan",
];

/// Re-detecta todas las herramientas del catálogo. Barato: sólo `command -v`,
/// nunca ejecuta las herramientas en sí.
pub fn detect_states(env: Env) -> Vec<ToolState> {
    catalog::ALL
        .iter()
        .map(|t| {
            if env.fixtures {
                let installed = FIXTURE_INSTALLED.contains(&t.id);
                ToolState {
                    id: t.id,
                    installed,
                    path: installed.then(|| format!("/usr/bin/{}", t.binary)),
                }
            } else {
                let path = which_path(t.binary);
                ToolState {
                    id: t.id,
                    installed: path.is_some(),
                    path,
                }
            }
        })
        .collect()
}

/// `true` si el binario está en el `PATH`.
pub fn which(binary: &str) -> bool {
    which_path(binary).is_some()
}

/// Ruta resuelta del binario en el `PATH`, o `None`.
pub fn which_path(binary: &str) -> Option<String> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|candidate| candidate.is_file())
        .map(|p| p.display().to_string())
}

/// Quita secuencias de escape ANSI (colores, estilos, movimientos de cursor)
/// de una línea de salida. Muchas herramientas colorean su salida y ratatui no
/// interpreta esas secuencias, así que sin esto aparecerían como texto literal
/// (p. ej. `[1m[34m`). Elimina CSI (`ESC[...`), OSC (`ESC]...`) y escapes de un
/// solo carácter; el resto del texto se conserva tal cual.
pub fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            // CSI: ESC [ ... hasta un byte final en @-~ (0x40..=0x7e).
            Some('[') => {
                chars.next();
                for f in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&f) {
                        break;
                    }
                }
            }
            // OSC: ESC ] ... terminado por BEL (0x07) o ST (ESC \).
            Some(']') => {
                chars.next();
                while let Some(f) = chars.next() {
                    if f == '\u{07}' {
                        break;
                    }
                    if f == '\x1b' && chars.peek() == Some(&'\\') {
                        chars.next();
                        break;
                    }
                }
            }
            // Cualquier otro escape de un solo carácter (ESC c): se descarta.
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    out
}

/// Construye la línea de argumentos a partir de `default_args` y el objetivo:
/// reemplaza `{target}`, y si la plantilla no lo lleva pero hay objetivo, lo
/// agrega al final. Devuelve los tokens separados por espacios (split simple,
/// suficiente para los flags del catálogo).
pub fn build_args(template: &str, target: &str) -> Vec<String> {
    let target = target.trim();
    let line = if template.contains("{target}") {
        template.replace("{target}", target)
    } else if target.is_empty() {
        template.to_string()
    } else {
        format!("{template} {target}")
    };
    line.split_whitespace().map(|s| s.to_string()).collect()
}

/// Comando de instalación sugerido para `tool` según el gestor detectado.
/// Devuelve `None` si no hay gestor conocido.
pub fn install_command(tool: &Tool, pm: Option<PackageManager>) -> Option<Vec<String>> {
    let pm = pm?;
    let pkg = match pm {
        PackageManager::Apt => tool.pkg_debian,
        PackageManager::Dnf | PackageManager::Yum | PackageManager::Zypper => tool.pkg_fedora,
        PackageManager::Pacman | PackageManager::Paru | PackageManager::Yay => tool.pkg_arch,
        PackageManager::Brew => tool.binary,
    };
    let base: Vec<String> = match pm {
        PackageManager::Apt => vec!["apt-get".into(), "install".into(), "-y".into(), pkg.into()],
        PackageManager::Dnf => vec!["dnf".into(), "install".into(), "-y".into(), pkg.into()],
        PackageManager::Yum => vec!["yum".into(), "install".into(), "-y".into(), pkg.into()],
        PackageManager::Zypper => {
            vec!["zypper".into(), "--non-interactive".into(), "install".into(), pkg.into()]
        }
        PackageManager::Pacman => {
            vec!["pacman".into(), "-S".into(), "--needed".into(), "--noconfirm".into(), pkg.into()]
        }
        PackageManager::Paru => vec!["paru".into(), "-S".into(), "--noconfirm".into(), pkg.into()],
        PackageManager::Yay => vec!["yay".into(), "-S".into(), "--noconfirm".into(), pkg.into()],
        PackageManager::Brew => return Some(vec!["brew".into(), "install".into(), pkg.into()]),
    };

    if is_root() {
        Some(base)
    } else {
        let mut with_sudo = vec!["sudo".to_string()];
        with_sudo.extend(base);
        Some(with_sudo)
    }
}

/// Vía de instalación alternativa para herramientas fuera de los repos
/// oficiales. El comando se ejecuta vía `sh -c` heredando la terminal (para que
/// un eventual `sudo` pueda pedir contraseña).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltKind {
    Go,
    Copr,
}

impl AltKind {
    pub fn label(&self) -> &'static str {
        match self {
            AltKind::Go => "go install",
            AltKind::Copr => "COPR",
        }
    }
}

/// Comando de instalación alternativo para `tool`, o `None` si no hay vía
/// alternativa aplicable. En Fedora (dnf/yum) prioriza COPR si la herramienta lo
/// tiene configurado; en el resto usa `go install` cuando hay ruta de módulo.
pub fn alt_install_command(tool: &Tool, pm: Option<PackageManager>) -> Option<(AltKind, Vec<String>)> {
    let on_dnf = matches!(pm, Some(PackageManager::Dnf) | Some(PackageManager::Yum));
    let sudo = if is_root() { "" } else { "sudo " };

    if on_dnf && let Some(copr) = tool.copr {
        let line = format!(
            "{sudo}dnf copr enable -y {copr} && {sudo}dnf install -y {pkg}",
            pkg = tool.pkg_fedora
        );
        return Some((AltKind::Copr, vec!["sh".into(), "-c".into(), line]));
    }

    if let Some(path) = tool.go_install {
        // Como root (caso típico dentro de un contenedor) instala en
        // /usr/local/bin para que quede en el PATH global; si no, al GOBIN del
        // usuario (~/go/bin).
        let line = if is_root() {
            format!("GOBIN=/usr/local/bin go install {path}")
        } else {
            format!("go install {path}")
        };
        return Some((AltKind::Go, vec!["sh".into(), "-c".into(), line]));
    }

    None
}

/// Gestor de paquetes del host, para armar comandos de instalación.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Zypper,
    Pacman,
    Paru,
    Yay,
    Brew,
}

impl PackageManager {
    pub fn label(&self) -> &'static str {
        match self {
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Yum => "yum",
            PackageManager::Zypper => "zypper",
            PackageManager::Pacman => "pacman",
            PackageManager::Paru => "paru",
            PackageManager::Yay => "yay",
            PackageManager::Brew => "brew",
        }
    }
}

/// Detecta el primer gestor disponible. Los helpers de AUR (paru/yay) van antes
/// que pacman porque varias herramientas del catálogo viven en AUR/BlackArch.
pub fn detect_package_manager() -> Option<PackageManager> {
    const CANDIDATES: [(&str, PackageManager); 8] = [
        ("apt-get", PackageManager::Apt),
        ("dnf", PackageManager::Dnf),
        ("yum", PackageManager::Yum),
        ("zypper", PackageManager::Zypper),
        ("paru", PackageManager::Paru),
        ("yay", PackageManager::Yay),
        ("pacman", PackageManager::Pacman),
        ("brew", PackageManager::Brew),
    ];
    CANDIDATES.iter().find(|(bin, _)| which(bin)).map(|(_, pm)| *pm)
}

/// Ejecuta un comando de remediación heredando la terminal (para que `sudo`
/// pueda pedir la contraseña). El llamador debe haber salido de la pantalla
/// alterna de ratatui antes.
pub fn run_install(command: &[String]) -> Result<bool, std::io::Error> {
    let Some((program, args)) = command.split_first() else {
        return Ok(false);
    };
    let status = std::process::Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(status.success())
}

pub fn command_line(command: &[String]) -> String {
    command.join(" ")
}

fn is_root() -> bool {
    // SAFETY: `geteuid` no toma punteros ni puede fallar.
    unsafe { libc::geteuid() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_quita_colores_y_conserva_texto() {
        assert_eq!(strip_ansi("\x1b[1m\x1b[34mPORT\x1b[0m 80/tcp"), "PORT 80/tcp");
        assert_eq!(strip_ansi("sin color"), "sin color");
        // OSC (título de terminal) terminado en BEL.
        assert_eq!(strip_ansi("\x1b]0;titulo\x07texto"), "texto");
    }

    #[test]
    fn build_args_reemplaza_el_objetivo() {
        assert_eq!(build_args("-sV {target}", "10.0.0.1"), vec!["-sV", "10.0.0.1"]);
    }

    #[test]
    fn build_args_agrega_el_objetivo_si_no_hay_marcador() {
        assert_eq!(build_args("-c 50 -n", "eth0"), vec!["-c", "50", "-n", "eth0"]);
    }

    #[test]
    fn build_args_sin_objetivo_deja_la_plantilla() {
        assert_eq!(build_args("audit system", ""), vec!["audit", "system"]);
    }

    #[test]
    fn alt_install_usa_go_cuando_hay_ruta_de_modulo() {
        let amass = catalog::by_id("amass").unwrap();
        // Sin dnf (p. ej. apt): debe elegir la vía go install.
        let (kind, cmd) = alt_install_command(amass, Some(PackageManager::Apt)).expect("vía go");
        assert_eq!(kind, AltKind::Go);
        assert!(cmd.last().unwrap().contains("go install github.com/owasp-amass"));
    }

    #[test]
    fn alt_install_none_para_herramienta_sin_vias() {
        let nmap = catalog::by_id("nmap").unwrap();
        assert!(alt_install_command(nmap, Some(PackageManager::Apt)).is_none());
    }
}
