use color_eyre::eyre::{Result, WrapErr, eyre};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::i18n::{self, Key, Lang};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollConfig {
    /// Cada cuántos segundos se re-detecta qué herramientas están instaladas.
    pub refresh_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub tick_rate_ms: u64,
    /// Timeout de cada comando ejecutado por la TUI, en segundos.
    pub command_timeout_secs: u64,
    #[serde(default)]
    pub language: Lang,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub poll: PollConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self { refresh_secs: 10 }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            command_timeout_secs: 120,
            language: Lang::default(),
        }
    }
}

impl Config {
    /// `Ok(None)` significa "no hay config en ningún lado": se usan los valores
    /// por defecto sin correr ningún asistente.
    pub fn load() -> Result<Option<Self>> {
        let Some(path) = Self::existing_path() else {
            return Ok(None);
        };
        let raw = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("no se pudo leer la config en {}", path.display()))?;
        let config: Config = toml::from_str(&raw)
            .wrap_err_with(|| format!("config inválida en {}", path.display()))?;
        Ok(Some(config))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let lang = self.ui.language;
        let path = Self::write_path(lang)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).wrap_err_with(|| {
                i18n::t(lang, Key::ConfigErrCreateDirTemplate)
                    .replacen("{}", &parent.display().to_string(), 1)
            })?;
        }

        let body = toml::to_string_pretty(self).wrap_err(i18n::t(lang, Key::ConfigErrSerialize))?;
        let contents = format!("# Generado por voxsecurity-tui.\n\n{body}");
        std::fs::write(&path, contents).wrap_err_with(|| {
            i18n::t(lang, Key::ConfigErrWriteTemplate).replacen("{}", &path.display().to_string(), 1)
        })?;
        Ok(path)
    }

    pub fn existing_path() -> Option<PathBuf> {
        let dev = PathBuf::from("config/voxsecurity.toml");
        if dev.exists() {
            return Some(dev);
        }
        let user = user_config_path()?;
        user.exists().then_some(user)
    }

    pub fn write_path(lang: Lang) -> Result<PathBuf> {
        if let Some(path) = Self::existing_path() {
            return Ok(path);
        }
        user_config_path().ok_or_else(|| eyre!(i18n::t(lang, Key::ConfigErrNoUserDir)))
    }
}

fn user_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "voxsecurity-tui")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    RefreshSecs,
    TimeoutSecs,
    Language,
}

impl ConfigField {
    pub const ALL: [ConfigField; 3] = [
        ConfigField::RefreshSecs,
        ConfigField::TimeoutSecs,
        ConfigField::Language,
    ];

    pub fn label(&self, lang: Lang) -> &'static str {
        match self {
            ConfigField::RefreshSecs => i18n::t(lang, Key::ConfigFieldRefreshLabel),
            ConfigField::TimeoutSecs => i18n::t(lang, Key::ConfigFieldTimeoutLabel),
            ConfigField::Language => i18n::t(lang, Key::ConfigFieldLanguageLabel),
        }
    }

    pub fn hint(&self, lang: Lang) -> &'static str {
        match self {
            ConfigField::RefreshSecs => i18n::t(lang, Key::ConfigFieldRefreshHint),
            ConfigField::TimeoutSecs => i18n::t(lang, Key::ConfigFieldTimeoutHint),
            ConfigField::Language => i18n::t(lang, Key::ConfigFieldLanguageHint),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigOutcome {
    None,
    Save(Box<Config>),
    Cancel,
}

/// Overlay de configuración (`c`): intervalos de refresco, timeout de comandos e
/// idioma. No hay credenciales que pedir.
#[derive(Debug, Clone)]
pub struct ConfigForm {
    pub focus: usize,
    pub refresh_secs: String,
    pub timeout_secs: String,
    pub language: Lang,
    pub error: Option<String>,
    tick_rate_ms: u64,
}

impl ConfigForm {
    pub fn new(config: &Config) -> Self {
        Self {
            focus: 0,
            refresh_secs: config.poll.refresh_secs.to_string(),
            timeout_secs: config.ui.command_timeout_secs.to_string(),
            language: config.ui.language,
            error: None,
            tick_rate_ms: config.ui.tick_rate_ms,
        }
    }

    pub fn field(&self) -> ConfigField {
        ConfigField::ALL[self.focus % ConfigField::ALL.len()]
    }

    pub fn display_value_of(&self, field: ConfigField) -> &str {
        match field {
            ConfigField::RefreshSecs => &self.refresh_secs,
            ConfigField::TimeoutSecs => &self.timeout_secs,
            ConfigField::Language => self.language.label(),
        }
    }

    fn value_mut(&mut self, field: ConfigField) -> &mut String {
        match field {
            ConfigField::RefreshSecs => &mut self.refresh_secs,
            ConfigField::TimeoutSecs => &mut self.timeout_secs,
            ConfigField::Language => unreachable!("Language no es un campo de texto"),
        }
    }

    pub fn on_key(&mut self, key: crossterm::event::KeyEvent) -> ConfigOutcome {
        use crossterm::event::KeyCode;

        self.error = None;
        match key.code {
            KeyCode::Esc => ConfigOutcome::Cancel,
            KeyCode::Up => {
                self.focus = (self.focus + ConfigField::ALL.len() - 1) % ConfigField::ALL.len();
                ConfigOutcome::None
            }
            KeyCode::Down => {
                self.focus = (self.focus + 1) % ConfigField::ALL.len();
                ConfigOutcome::None
            }
            KeyCode::Left | KeyCode::Right if self.field() == ConfigField::Language => {
                self.language = self.language.toggle();
                ConfigOutcome::None
            }
            KeyCode::Backspace if self.field() != ConfigField::Language => {
                let field = self.field();
                self.value_mut(field).pop();
                ConfigOutcome::None
            }
            KeyCode::Char(c) if c.is_ascii_digit() && self.field() != ConfigField::Language => {
                let field = self.field();
                self.value_mut(field).push(c);
                ConfigOutcome::None
            }
            KeyCode::Enter => self.save(),
            _ => ConfigOutcome::None,
        }
    }

    fn save(&mut self) -> ConfigOutcome {
        let lang = self.language;
        let parse = |s: &str, label: &str| -> Result<u64, String> {
            s.parse::<u64>()
                .map_err(|_| i18n::t(lang, Key::ConfigErrInvalidTemplate).replacen("{}", label, 1))
                .and_then(|v| {
                    if v == 0 {
                        Err(i18n::t(lang, Key::ConfigErrZeroTemplate).replacen("{}", label, 1))
                    } else {
                        Ok(v)
                    }
                })
        };

        let refresh = match parse(&self.refresh_secs, ConfigField::RefreshSecs.label(lang)) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(e);
                return ConfigOutcome::None;
            }
        };
        let timeout = match parse(&self.timeout_secs, ConfigField::TimeoutSecs.label(lang)) {
            Ok(v) => v,
            Err(e) => {
                self.error = Some(e);
                return ConfigOutcome::None;
            }
        };

        let config = Config {
            poll: PollConfig { refresh_secs: refresh },
            ui: UiConfig {
                tick_rate_ms: self.tick_rate_ms,
                command_timeout_secs: timeout,
                language: self.language,
            },
        };
        ConfigOutcome::Save(Box::new(config))
    }
}
