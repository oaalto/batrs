use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserSettings {
    pub entries: Vec<SettingEntry>,
}

impl UserSettings {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }
}

#[derive(Debug)]
pub enum SettingsError {
    EmptyKey,
    DuplicateKey(String),
    MissingEquals(String),
    InvalidValue(String),
    Io(io::Error),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::EmptyKey => write!(f, "settings key is empty"),
            SettingsError::DuplicateKey(key) => write!(f, "duplicate settings key: {key}"),
            SettingsError::MissingEquals(line) => {
                write!(f, "settings line missing '=': {line}")
            }
            SettingsError::InvalidValue(value) => {
                write!(f, "settings value must be quoted: {value}")
            }
            SettingsError::Io(err) => write!(f, "settings IO error: {err}"),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(err: io::Error) -> Self {
        SettingsError::Io(err)
    }
}

struct SettingDefinition {
    key: &'static str,
    default: &'static str,
}

const SETTINGS_DEFS: &[SettingDefinition] = &[SettingDefinition {
    key: "rig",
    default: "",
}];

#[derive(Debug, Default)]
pub struct ConfigManager {
    base_dir: PathBuf,
    base_config: Option<String>,
    user_config: Option<String>,
    user_config_path: Option<PathBuf>,
}

impl ConfigManager {
    pub fn new() -> io::Result<Self> {
        let home = env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
        let base_dir = PathBuf::from(home).join(".batrs");
        Ok(Self {
            base_dir,
            base_config: None,
            user_config: None,
            user_config_path: None,
        })
    }

    pub fn init_base(&mut self) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let base_config_path = self.base_dir.join("batrs.toml");
        ensure_empty_file(&base_config_path)?;
        self.base_config = Some(fs::read_to_string(&base_config_path)?);
        Ok(())
    }

    pub fn load_user(&mut self, player_name: &str) -> io::Result<()> {
        let safe_name = sanitize_name(player_name);
        let player_dir = self.base_dir.join(&safe_name);
        fs::create_dir_all(&player_dir)?;
        let player_config_path = player_dir.join(format!("{safe_name}.toml"));
        ensure_empty_file(&player_config_path)?;
        self.user_config = Some(fs::read_to_string(&player_config_path)?);
        self.user_config_path = Some(player_config_path);
        Ok(())
    }

    pub fn user_guilds(&self) -> Option<Vec<String>> {
        let config = self.user_config.as_deref()?;
        parse_guilds(config)
    }

    pub fn user_settings(&mut self) -> Result<UserSettings, SettingsError> {
        let config = self.user_config.as_deref().unwrap_or("");
        let parsed = parse_settings(config)?;
        let (entries, changed) = normalize_settings_entries(parsed.unwrap_or_default());
        if changed {
            self.persist_user_config(update_settings_config(config, &entries))?;
        }
        Ok(UserSettings { entries })
    }

    pub fn save_user_guilds(&mut self, guilds: &[String]) -> io::Result<()> {
        let Some(path) = self.user_config_path.as_ref() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "user config path not set",
            ));
        };
        let updated = update_guilds_config(self.user_config.as_deref().unwrap_or(""), guilds);
        fs::write(path, updated.as_bytes())?;
        self.user_config = Some(updated);
        Ok(())
    }

    pub fn save_user_settings(&mut self, settings: &UserSettings) -> io::Result<()> {
        let Some(path) = self.user_config_path.as_ref() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "user config path not set",
            ));
        };
        let (entries, _) = normalize_settings_entries(settings.entries.clone());
        let updated = update_settings_config(self.user_config.as_deref().unwrap_or(""), &entries);
        fs::write(path, updated.as_bytes())?;
        self.user_config = Some(updated);
        Ok(())
    }

    fn persist_user_config(&mut self, updated: String) -> io::Result<()> {
        let Some(path) = self.user_config_path.as_ref() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "user config path not set",
            ));
        };
        fs::write(path, updated.as_bytes())?;
        self.user_config = Some(updated);
        Ok(())
    }
}

fn ensure_empty_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err),
    }
}

pub(crate) fn sanitize_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' => c,
            _ => '_',
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "player".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_guilds(config: &str) -> Option<Vec<String>> {
    for line in config.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if !line.starts_with("guilds") {
            continue;
        }
        let eq_index = match line.find('=') {
            Some(index) => index,
            None => return Some(Vec::new()),
        };
        let value = line[eq_index + 1..].trim();
        let start = match value.find('[') {
            Some(index) => index,
            None => return Some(Vec::new()),
        };
        let end = match value.rfind(']') {
            Some(index) => index,
            None => return Some(Vec::new()),
        };
        let inner = &value[start + 1..end];
        let guilds = inner
            .split(',')
            .map(|entry| entry.trim().trim_matches(|c| c == '"' || c == '\''))
            .filter(|entry| !entry.is_empty())
            .map(|entry| entry.to_string())
            .collect::<Vec<String>>();
        return Some(guilds);
    }
    None
}

fn parse_settings(config: &str) -> Result<Option<Vec<SettingEntry>>, SettingsError> {
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut in_settings = false;
    let mut found = false;

    for line in config.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if trimmed.is_empty() {
            continue;
        }
        if is_table_header(trimmed) {
            in_settings = trimmed == "[settings]";
            if in_settings {
                found = true;
            }
            continue;
        }
        if !in_settings {
            continue;
        }

        let (key, value) = trimmed
            .split_once('=')
            .ok_or_else(|| SettingsError::MissingEquals(trimmed.to_string()))?;
        let key = key.trim();
        if key.is_empty() {
            return Err(SettingsError::EmptyKey);
        }
        if !seen.insert(key.to_string()) {
            return Err(SettingsError::DuplicateKey(key.to_string()));
        }
        let value = parse_settings_value(value.trim())?;
        entries.push(SettingEntry {
            key: key.to_string(),
            value,
        });
    }

    if found { Ok(Some(entries)) } else { Ok(None) }
}

fn normalize_settings_entries(entries: Vec<SettingEntry>) -> (Vec<SettingEntry>, bool) {
    let mut known = HashMap::new();
    let mut extras = Vec::new();
    for entry in entries {
        if SETTINGS_DEFS.iter().any(|def| def.key == entry.key) {
            known.insert(entry.key, entry.value);
        } else {
            extras.push(entry);
        }
    }

    let mut changed = false;
    let mut normalized = Vec::new();
    for def in SETTINGS_DEFS {
        if let Some(value) = known.remove(def.key) {
            normalized.push(SettingEntry {
                key: def.key.to_string(),
                value,
            });
        } else {
            normalized.push(SettingEntry {
                key: def.key.to_string(),
                value: def.default.to_string(),
            });
            changed = true;
        }
    }
    normalized.extend(extras);
    (normalized, changed)
}

fn parse_settings_value(value: &str) -> Result<String, SettingsError> {
    let value = value.trim();
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        let inner = &value[1..value.len().saturating_sub(1)];
        return Ok(inner.replace("\\\"", "\""));
    }
    if value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2 {
        let inner = &value[1..value.len().saturating_sub(1)];
        return Ok(inner.replace("\\'", "'"));
    }
    Err(SettingsError::InvalidValue(value.to_string()))
}

fn update_settings_config(existing: &str, entries: &[SettingEntry]) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_settings = false;
    for line in existing.lines() {
        let trimmed = line.split('#').next().unwrap_or("").trim();
        if is_table_header(trimmed) {
            in_settings = trimmed == "[settings]";
            if in_settings {
                continue;
            }
        }
        if in_settings {
            continue;
        }
        lines.push(line.to_string());
    }

    if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
        lines.push(String::new());
    }
    lines.push("[settings]".to_string());
    for entry in entries {
        let formatted = entry.value.replace('"', "\\\"");
        lines.push(format!("{} = \"{formatted}\"", entry.key));
    }

    let mut output = lines.join("\n");
    output.push('\n');
    output
}

fn is_table_header(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}

fn update_guilds_config(existing: &str, guilds: &[String]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("guilds") {
            continue;
        }
        lines.push(line.to_string());
    }

    let formatted = guilds
        .iter()
        .map(|guild| format!("\"{}\"", guild.replace('"', "\\\"")))
        .collect::<Vec<String>>()
        .join(", ");
    lines.push(format!("guilds = [{formatted}]"));

    let mut output = lines.join("\n");
    output.push('\n');
    output
}
