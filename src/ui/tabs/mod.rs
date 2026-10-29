pub mod catalog;
pub mod logs;
pub mod overview;
pub mod scan;
pub mod syscheck;

use crate::i18n::{self, Key, Lang};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Overview,
    Catalog,
    Scan,
    Logs,
    SystemCheck,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Overview,
        Tab::Catalog,
        Tab::Scan,
        Tab::Logs,
        Tab::SystemCheck,
    ];

    pub fn title(&self, lang: Lang) -> &'static str {
        let key = match self {
            Tab::Overview => Key::TabOverview,
            Tab::Catalog => Key::TabCatalog,
            Tab::Scan => Key::TabScan,
            Tab::Logs => Key::TabLogs,
            Tab::SystemCheck => Key::TabSystemCheck,
        };
        i18n::t(lang, key)
    }

    pub fn index(&self) -> usize {
        Self::ALL.iter().position(|t| t == self).expect("Tab siempre está en Tab::ALL")
    }

    pub fn from_index(index: usize) -> Option<Tab> {
        Self::ALL.get(index).copied()
    }

    pub fn next(self) -> Self {
        Self::ALL[(self.index() + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        Self::ALL[(self.index() + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}
