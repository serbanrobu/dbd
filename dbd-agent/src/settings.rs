use config::{Config, ConfigError, Environment, File};
use dirs::home_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub api_keys: HashMap<String, String>,
    pub port: u16,
    pub address: String,
    pub connections: HashMap<String, Connection>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Connection {
    pub kind: ConnectionKind,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub dbname: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionKind {
    Postgres,
    MySql,
}

pub fn configure(path: Option<PathBuf>) -> Result<Settings, ConfigError> {
    let mut settings = Config::default();
    settings.merge(File::with_name(
        path.or_else(|| home_dir().map(|h| h.join(".dbd-agent.toml")))
            .unwrap()
            .to_str()
            .unwrap(),
    ))?;
    settings.merge(Environment::new())?;
    settings.try_into()
}
