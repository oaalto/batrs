use crate::config::sanitize_name;
use chrono::Local;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub struct RawLogger {
    base_dir: PathBuf,
    file: Option<File>,
    path: Option<PathBuf>,
}

impl RawLogger {
    pub fn new() -> io::Result<Self> {
        let home = env::var_os("HOME")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME not set"))?;
        Ok(Self {
            base_dir: PathBuf::from(home).join(".batrs").join("raw_logs"),
            file: None,
            path: None,
        })
    }

    pub fn is_enabled(&self) -> bool {
        self.file.is_some()
    }

    pub fn enable(&mut self, player_name: Option<&str>) -> io::Result<&Path> {
        if self.file.is_none() {
            fs::create_dir_all(&self.base_dir)?;
            let timestamp = Local::now().format("%Y%m%d-%H%M%S");
            let player = player_name
                .map(sanitize_name)
                .filter(|name| !name.is_empty())
                .unwrap_or_else(|| "unknown".to_string());
            let path = self.base_dir.join(format!("{player}-{timestamp}.raw.log"));
            let file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)?;
            self.file = Some(file);
            self.path = Some(path);
        }

        self.path
            .as_deref()
            .ok_or_else(|| io::Error::other("raw log path missing"))
    }

    pub fn disable(&mut self) {
        self.file = None;
        self.path = None;
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.write_all(bytes)?;
            file.flush()?;
        }
        Ok(())
    }
}
