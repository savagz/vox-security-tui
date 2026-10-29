use crate::i18n::{self, Key, Lang};
use crate::theme::Status;
use crate::tools::{self, Env, PackageManager};

/// Una fila de la verificación del sistema. A diferencia de otras TUIs de la
/// familia VOXPBX, acá ningún chequeo es bloqueante: la app es un orquestador y
/// puede arrancar aunque falten herramientas (se instalan desde el Catálogo).
#[derive(Debug, Clone)]
pub struct DepCheck {
    pub label: &'static str,
    pub detail: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct DepReport {
    pub package_manager: Option<PackageManager>,
    pub checks: Vec<DepCheck>,
}

impl DepReport {
    /// Ningún chequeo en rojo: nada reclama atención inmediata.
    pub fn all_ok(&self) -> bool {
        self.checks.iter().all(|c| c.status != Status::Error)
    }

    pub fn get(&self, index: usize) -> Option<&DepCheck> {
        self.checks.get(index)
    }
}

/// Corre los chequeos. Se dispara sólo por tecla del usuario (abrir el wizard,
/// `r` en la pestaña), nunca en el loop caliente.
pub fn probe(lang: Lang, env: Env) -> DepReport {
    let t = |k: Key| i18n::t(lang, k);
    let pm = tools::detect_package_manager();

    let states = tools::detect_states(env);
    let installed = states.iter().filter(|s| s.installed).count();
    let total = states.len();

    let mut checks = Vec::with_capacity(4);

    checks.push(DepCheck {
        label: t(Key::DepPackageManagerLabel),
        detail: match pm {
            Some(pm) => t(Key::DepPackageManagerOkTemplate).replacen("{}", pm.label(), 1),
            None => t(Key::DepPackageManagerMissing).to_string(),
        },
        status: if pm.is_some() { Status::Ok } else { Status::Warning },
    });

    checks.push(DepCheck {
        label: t(Key::DepToolsLabel),
        detail: if installed > 0 {
            t(Key::DepToolsOkTemplate)
                .replacen("{}", &installed.to_string(), 1)
                .replacen("{}", &total.to_string(), 1)
        } else {
            t(Key::DepToolsNoneTemplate).replacen("{}", &total.to_string(), 1)
        },
        status: if installed > 0 { Status::Ok } else { Status::Warning },
    });

    checks.push(DepCheck {
        label: t(Key::DepPrivilegeLabel),
        detail: if is_root() {
            t(Key::DepPrivilegeRoot).to_string()
        } else {
            t(Key::DepPrivilegeUser).to_string()
        },
        status: Status::Info,
    });

    checks.push(DepCheck {
        label: t(Key::DepFixturesLabel),
        detail: if env.fixtures {
            t(Key::DepFixturesOn).to_string()
        } else {
            t(Key::DepFixturesOff).to_string()
        },
        status: if env.fixtures { Status::Info } else { Status::Ok },
    });

    DepReport { package_manager: pm, checks }
}

fn is_root() -> bool {
    // SAFETY: `geteuid` no toma punteros ni puede fallar.
    unsafe { libc::geteuid() == 0 }
}
