use config::{Config, ConfigError, Environment, File};
use dirs::home_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub api_key: String,
    pub port: u16,
    pub address: String,
    pub databases: Databases,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Database {
    pub connection: Connection,
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub username: String,
    pub password: String,
    pub exclude_table_data: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    Postgres,
    MySql,
}

pub type Databases = HashMap<String, Database>;

pub fn configure(path: Option<PathBuf>) -> Result<Settings, ConfigError> {
    let mut settings = Config::default();
    settings.merge(File::with_name(
        path.or_else(|| home_dir().map(|h| h.join(".dbd-agent")))
            .unwrap()
            .to_str()
            .unwrap(),
    ))?;
    settings.merge(Environment::new())?;
    settings.try_into()
}
