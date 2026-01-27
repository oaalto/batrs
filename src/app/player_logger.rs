use crate::config::sanitize_name;
use chrono::Local;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

pub struct PlayerLogger {
    base_dir: PathBuf,
    player_name: Option<String>,
    current_date: Option<String>,
    file: Option<File>,
}

impl PlayerLogger {
    pub fn new() -> io::Result<Self> {
        let home = env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
        let base_dir = PathBuf::from(home).join(".batrs");
        Ok(Self {
            base_dir,
            player_name: None,
            current_date: None,
            file: None,
        })
    }

    pub fn set_player_name(&mut self, name: &str) {
        let sanitized = sanitize_name(name);
        if self.player_name.as_deref() == Some(sanitized.as_str()) {
            return;
        }
        self.player_name = Some(sanitized);
        self.current_date = None;
        self.file = None;
    }

    pub fn log_line(&mut self, line: &str) -> io::Result<()> {
        let Some(player_name) = self.player_name.clone() else {
            return Ok(());
        };
        let date = Local::now().format("%Y%m%d").to_string();
        if self.current_date.as_deref() != Some(date.as_str()) || self.file.is_none() {
            let log_dir = self.base_dir.join(&player_name).join("logs");
            fs::create_dir_all(&log_dir)?;
            let log_path = log_dir.join(format!("{date}.log"));
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;
            self.file = Some(file);
            self.current_date = Some(date);
        }

        if let Some(file) = self.file.as_mut() {
            writeln!(file, "{line}")?;
        }
        Ok(())
    }
}
