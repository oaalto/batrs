use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct ConfigManager {
    base_dir: PathBuf,
    base_config: Option<String>,
    user_config: Option<String>,
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
        Ok(())
    }
}

fn ensure_empty_file(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::File::create(path)?;
    }
    Ok(())
}

fn sanitize_name(name: &str) -> String {
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
