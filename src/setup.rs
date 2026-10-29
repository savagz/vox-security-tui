use crossterm::event::{KeyCode, KeyEvent};

use crate::deps::{self, DepReport};
use crate::i18n::Lang;
use crate::tools::Env;

/// `Gate` es el wizard a pantalla completa que corre antes del tablero. El mismo
/// estado se reutiliza dentro de la pestaña `SYSTEM CHECK`, pero ahí la
/// navegación la maneja `App` directamente, sin pasar por `on_key`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemCheckMode {
    Gate,
}

#[derive(Debug, Clone)]
pub enum SystemCheckOutcome {
    None,
    Recheck,
    /// Sólo en modo `Gate`: entrar al tablero igual.
    Continue,
    /// Sólo en modo `Gate`: cancelar y terminar el proceso.
    Cancel,
}

#[derive(Debug, Clone)]
pub struct SystemCheckState {
    pub report: DepReport,
    pub selected: usize,
    pub lang: Lang,
    pub env: Env,
}

impl SystemCheckState {
    pub fn new(lang: Lang, env: Env) -> Self {
        Self {
            report: deps::probe(lang, env),
            selected: 0,
            lang,
            env,
        }
    }

    pub fn recheck(&mut self) {
        self.report = deps::probe(self.lang, self.env);
    }

    /// Cambia el idioma y re-corre el probe para regenerar los textos.
    pub fn set_lang(&mut self, lang: Lang) {
        if self.lang != lang {
            self.lang = lang;
            self.recheck();
        }
    }

    pub fn on_key(&mut self, key: KeyEvent, mode: SystemCheckMode) -> SystemCheckOutcome {
        match key.code {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                SystemCheckOutcome::None
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.report.checks.len().saturating_sub(1));
                SystemCheckOutcome::None
            }
            KeyCode::Char('r') => SystemCheckOutcome::Recheck,
            KeyCode::Char('a') if mode == SystemCheckMode::Gate => SystemCheckOutcome::Continue,
            KeyCode::Esc if mode == SystemCheckMode::Gate => SystemCheckOutcome::Cancel,
            _ => SystemCheckOutcome::None,
        }
    }
}
