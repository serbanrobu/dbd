use config::{Config, ConfigError, File};
use dirs::home_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use surf::Url;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub agents: HashMap<String, Agent>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Agent {
    pub url: Url,
    pub api_key: String,
}

pub fn configure(path: Option<PathBuf>) -> Result<Option<Settings>, ConfigError> {
    let path = path.or_else(|| {
        home_dir()
            .map(|h| h.join(".dbd.toml"))
            .filter(|p| p.exists())
    });
    let filename = match &path {
        Some(p) => p.to_str().unwrap(),
        _ => return Ok(None),
    };
    let mut settings = Config::default();
    settings.merge(File::with_name(filename))?;
    settings.try_into()
}
