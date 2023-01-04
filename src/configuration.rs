use std::time::Duration;

use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::domain::SubscriberEmail;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub name: String,
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
    Use either `development` or `production`.",
                other
            )),
        }
    }
}

pub fn get() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");

    let environment: Environment = std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse ENVIRONMENT");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let settings = Config::builder()
        // configuration/base.yaml
        .add_source(File::from(configuration_directory.join("base.yaml")))
        // merge with configuration/development.yaml or configuration/production.yaml
        .add_source(File::from(
            configuration_directory.join(environment_filename),
        ))
        // environment variables that are injected from the PaaS or server when deployed
        .add_source(config::Environment::default())
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

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    sender_email: String,
    pub authorization_token: Secret<String>,
    timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}
