use std::{io::Error, net::TcpListener, time::Duration};

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{self, DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{health::health, subscriptions::subscribe},
};

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let pool = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    let config = configuration::get().expect("Failed to read configuration.");
    info!(
        "Starting on {}:{}",
        config.application.host, config.application.port,
    );

    Ok(server)
}

#[must_use]
pub fn get_connection_pool(database: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(60))
        .connect_lazy(database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres")
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: &Settings) -> Result<Self, Error> {
        // Database
        let connection_pool = get_connection_pool(&configuration.database);
        // Migrate database in case the database is new
        // Must be idempotent
        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to run database migrations");

        // Email client
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Failed to get email");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url.to_string(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            timeout,
        );

        // Application
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().expect("Failed to bind port").port();
        let server = run(listener, connection_pool, email_client)?;

        Ok(Self { port, server })
    }

    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// A more expressive name that makes it clear that
    /// this function only returns when the application is stopped.
    pub async fn run_until_stopped(self) -> Result<(), Error> {
        self.server.await
    }
}
