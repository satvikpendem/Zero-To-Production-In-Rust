use std::{io::stdout, net::TcpListener, time::Duration};

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration,
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), stdout);
    init_subscriber(subscriber);

    // Application
    let configuration = configuration::get().expect("Failed to read configuration");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;

    // Database
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(60))
        .connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run database migrations");

    // Email client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Failed to get email");
    let email_client = EmailClient::new(configuration.email_client.base_url, sender_email);

    run(listener, connection_pool, email_client)?.await?;
    Ok(())
}
