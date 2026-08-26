use std::{fs, io, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(serde_yaml::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not read config file: {}", e),
            ConfigError::Parse(e) => write!(f, "invalid config: {}", e),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(e: serde_yaml::Error) -> Self {
        ConfigError::Parse(e)
    }
}
