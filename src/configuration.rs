use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("configuration.yaml", FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        let Self {
            username,
            password,
            host,
            port,
            database_name,
        } = self;
        format!("postgres://{username}:{password}@{host}:{port}/{database_name}")
    }
}
