use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PlayerToml {
    /// When omitted from file, guild lists do not override defaults at login (same as legacy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guilds: Option<Vec<String>>,
    #[serde(default)]
    pub settings: SettingsTable,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SettingsTable {
    #[serde(default)]
    pub rig: String,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
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
    user_config_path: Option<PathBuf>,
    player_config: Option<PlayerToml>,
}

impl ConfigManager {
    pub fn new() -> io::Result<Self> {
        let home = env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
        let base_dir = PathBuf::from(home).join(".batrs");
        Ok(Self {
            base_dir,
            base_config: None,
            user_config_path: None,
            player_config: None,
        })
    }

    pub fn init_base(&mut self) -> io::Result<()> {
        fs::create_dir_all(&self.base_dir)?;
        let base_config_path = self.base_dir.join("batrs.toml");
        ensure_empty_file(&base_config_path)?;
        self.base_config = Some(fs::read_to_string(&base_config_path)?);
        Ok(())
    }

    #[allow(clippy::collapsible_if)]
    pub fn load_user(&mut self, player_name: &str) -> io::Result<()> {
        let safe_name = sanitize_name(player_name);
        let player_dir = self.base_dir.join(&safe_name);
        fs::create_dir_all(&player_dir)?;
        let player_config_path = player_dir.join(format!("{safe_name}.toml"));
        ensure_empty_file(&player_config_path)?;
        let contents = fs::read_to_string(&player_config_path)?;
        self.user_config_path = Some(player_config_path.clone());

        let player = match parse_or_migrate(&contents) {
            Ok((mut player, legacy_used)) => {
                let normalized = normalize_player_config(&mut player);
                if legacy_used || normalized {
                    if let Err(err) = persist_player_to_path(&player_config_path, &player) {
                        eprintln!("failed to rewrite migrated player config: {err}");
                    }
                }
                player
            }
            Err(err) => {
                eprintln!("invalid player config (using defaults): {err}");
                PlayerToml::default()
            }
        };

        self.player_config = Some(player);

        Ok(())
    }

    pub fn user_guilds(&self) -> Option<Vec<String>> {
        self.player_config.as_ref()?.guilds.clone()
    }

    pub fn user_settings(&mut self) -> Result<UserSettings, SettingsError> {
        let Some(player) = self.player_config.as_mut() else {
            return Ok(UserSettings::default());
        };

        let changed = normalize_player_config(player);
        if changed && let Some(path) = self.user_config_path.as_ref() {
            persist_player_to_path(path, player).map_err(SettingsError::Io)?;
        }

        Ok(player_to_user_settings(player))
    }

    pub fn save_user_guilds(&mut self, guilds: &[String]) -> io::Result<()> {
        let Some(path) = self.user_config_path.as_ref() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "user config path not set",
            ));
        };
        let Some(player) = self.player_config.as_mut() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "player config not loaded",
            ));
        };
        player.guilds = Some(guilds.to_vec());
        persist_player_to_path(path, player)
    }

    pub fn save_user_settings(&mut self, settings: &UserSettings) -> io::Result<()> {
        let Some(path) = self.user_config_path.as_ref() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "user config path not set",
            ));
        };
        let Some(player) = self.player_config.as_mut() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "player config not loaded",
            ));
        };
        let (entries, _) = normalize_settings_entries(settings.entries.clone());
        player.settings = settings_table_from_entries(&entries);
        persist_player_to_path(path, player)
    }
}

fn persist_player_to_path(path: &Path, player: &PlayerToml) -> io::Result<()> {
    let rendered = toml::to_string_pretty(player)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let mut out = rendered;
    if !out.ends_with('\n') {
        out.push('\n');
    }
    fs::write(path, out.as_bytes())
}

fn parse_or_migrate(raw: &str) -> Result<(PlayerToml, bool), SettingsError> {
    match toml::from_str::<PlayerToml>(raw.trim()) {
        Ok(player) => Ok((player, false)),
        Err(_) => migrate_legacy_config(raw).map(|player| (player, true)),
    }
}

fn migrate_legacy_config(raw: &str) -> Result<PlayerToml, SettingsError> {
    let parsed_settings = parse_settings(raw)?;
    let guilds_migrated = parsed_settings
        .as_ref()
        .and_then(|section| section.guilds_from_settings_table.clone());
    let entry_vec = parsed_settings
        .map(|section| section.entries)
        .unwrap_or_default();

    let guilds = match guilds_migrated {
        Some(migrated) => {
            let top_level = parse_guilds(raw).unwrap_or_default();
            let merged = if !top_level.is_empty() {
                top_level
            } else {
                migrated
            };
            Some(merged)
        }
        None => parse_guilds(raw),
    };

    let (entries, _) = normalize_settings_entries(entry_vec);
    Ok(PlayerToml {
        guilds,
        settings: settings_table_from_entries(&entries),
    })
}

fn player_to_user_settings(player: &PlayerToml) -> UserSettings {
    let mut entries = Vec::new();
    entries.push(SettingEntry {
        key: "rig".to_string(),
        value: player.settings.rig.clone(),
    });
    let mut keys: Vec<String> = player.settings.extra.keys().cloned().collect();
    keys.sort();
    for key in keys {
        if let Some(value) = player.settings.extra.get(&key) {
            entries.push(SettingEntry {
                key,
                value: value.clone(),
            });
        }
    }
    UserSettings { entries }
}

fn settings_table_from_entries(entries: &[SettingEntry]) -> SettingsTable {
    let mut rig = String::new();
    let mut extra = HashMap::new();
    for entry in entries {
        if entry.key == "rig" {
            rig.clone_from(&entry.value);
        } else {
            extra.insert(entry.key.clone(), entry.value.clone());
        }
    }
    SettingsTable { rig, extra }
}

fn normalize_player_config(player: &mut PlayerToml) -> bool {
    let entries = player_to_user_settings(player).entries;
    let (normalized, changed) = normalize_settings_entries(entries);
    if changed {
        player.settings = settings_table_from_entries(&normalized);
    }
    changed
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

fn parse_bracket_string_array(value: &str) -> Option<Vec<String>> {
    let value = value.trim();
    let start = value.find('[')?;
    let end = value.rfind(']')?;
    if end <= start {
        return None;
    }
    let inner = &value[start + 1..end];
    let guilds = inner
        .split(',')
        .map(|entry| entry.trim().trim_matches(|c| c == '"' || c == '\''))
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_string())
        .collect::<Vec<String>>();
    Some(guilds)
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
        return Some(parse_bracket_string_array(value).unwrap_or_default());
    }
    None
}

struct ParsedSettingsSection {
    entries: Vec<SettingEntry>,
    guilds_from_settings_table: Option<Vec<String>>,
}

fn parse_settings(config: &str) -> Result<Option<ParsedSettingsSection>, SettingsError> {
    let mut entries = Vec::new();
    let mut guilds_from_settings_table: Option<Vec<String>> = None;
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

        if key == "guilds" {
            let raw = value.trim();
            if let Some(list) = parse_bracket_string_array(raw) {
                guilds_from_settings_table = Some(list);
                continue;
            }
            match parse_legacy_settings_scalar(raw) {
                Ok(s) => {
                    if let Some(list) = parse_bracket_string_array(&s) {
                        guilds_from_settings_table = Some(list);
                    } else if !s.is_empty() {
                        guilds_from_settings_table = Some(vec![s]);
                    }
                }
                Err(_) => return Err(SettingsError::InvalidValue(raw.to_string())),
            }
            continue;
        }

        let value = parse_legacy_settings_scalar(value.trim())?;
        entries.push(SettingEntry {
            key: key.to_string(),
            value,
        });
    }

    if found {
        Ok(Some(ParsedSettingsSection {
            entries,
            guilds_from_settings_table,
        }))
    } else {
        Ok(None)
    }
}

fn parse_legacy_settings_scalar(value: &str) -> Result<String, SettingsError> {
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

fn is_table_header(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_player_toml() {
        let original = PlayerToml {
            guilds: Some(vec!["reaver".to_string(), "monk".to_string()]),
            settings: SettingsTable {
                rig: "bag".to_string(),
                extra: HashMap::from([("note".to_string(), "hello".to_string())]),
            },
        };
        let text = toml::to_string_pretty(&original).unwrap();
        let parsed: PlayerToml = toml::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn serde_omitted_guilds_deserializes_to_none() {
        let text = r#"
[settings]
rig = "x"
"#;
        let parsed: PlayerToml = toml::from_str(text).unwrap();
        assert_eq!(parsed.guilds, None);
    }

    #[test]
    fn parse_settings_accepts_bracket_guilds_inside_settings_table() {
        let config = r#"
[settings]
guilds = ["animist", "monk"]
rig = "satchel"
"#;
        let parsed = parse_settings(config).unwrap().unwrap();
        assert_eq!(
            parsed.guilds_from_settings_table,
            Some(vec!["animist".to_string(), "monk".to_string()])
        );
        assert_eq!(parsed.entries.len(), 1);
        assert_eq!(parsed.entries[0].key, "rig");
        assert_eq!(parsed.entries[0].value, "satchel");
    }

    #[test]
    fn parse_settings_accepts_bracket_guilds_inside_single_quoted_value() {
        let config = r#"
[settings]
guilds = '["animist", "monk"]'
"#;
        let parsed = parse_settings(config).unwrap().unwrap();
        assert_eq!(
            parsed.guilds_from_settings_table,
            Some(vec!["animist".to_string(), "monk".to_string()])
        );
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn parse_guilds_returns_empty_when_brackets_missing() {
        let config = "guilds = not-an-array";
        assert_eq!(parse_guilds(config), Some(Vec::new()));
    }

    #[test]
    fn parse_bracket_string_array_trims_quotes() {
        assert_eq!(
            parse_bracket_string_array(r#"["reaver", 'monk']"#),
            Some(vec!["reaver".to_string(), "monk".to_string()])
        );
    }

    #[test]
    fn migrate_legacy_misplaced_guilds_produces_valid_toml() {
        let raw = r#"
[settings]
guilds = ["animist", "monk"]
rig = "sack"
"#;
        let legacy = migrate_legacy_config(raw).unwrap();
        assert_eq!(
            legacy.guilds,
            Some(vec!["animist".to_string(), "monk".to_string()])
        );
        assert_eq!(legacy.settings.rig, "sack");
        let round = toml::to_string_pretty(&legacy).unwrap();
        let _: PlayerToml = toml::from_str(&round).unwrap();
    }

    #[test]
    fn parse_or_migrate_strict_then_roundtrip() {
        let raw = r#"guilds = ["disciple"]

[settings]
rig = "pack"
"#;
        let (player, legacy) = parse_or_migrate(raw).unwrap();
        assert!(!legacy);
        assert_eq!(player.guilds, Some(vec!["disciple".to_string()]));
        assert_eq!(player.settings.rig, "pack");
    }
}
