use config::{Config, ConfigError, File, FileFormat};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
}

pub fn get() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("configuration.yaml", FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    #[must_use]
    pub fn connection_string(&self) -> Secret<String> {
        let Self {
            username,
            password,
            host,
            port,
            name,
        } = self;
        Secret::new(format!(
            "postgres://{username}:{}@{host}:{port}/{name}",
            password.expose_secret()
        ))
    }

    #[must_use]
    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
