use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
}

fn ensure_empty_file(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::File::create(path)?;
    }
    Ok(())
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
