use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Executor, PgPool};
use std::{
    io::{sink, stdout},
    net::TcpListener,
};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    email_client::EmailClient,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber_name = "test".to_string();
    let default_filter_level = "info".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub database_pool: PgPool,
    pub database_name: String,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Application
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut configuration = configuration::get().expect("Failed to read configuration.");

    // Database
    configuration.database.name = Uuid::new_v4().to_string();
    let database_pool = configure_database(&configuration.database).await;

    // Email Client
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let server =
        run(listener, database_pool.clone(), email_client).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp {
        address,
        database_pool,
        database_name: configuration.database.name,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let connection = PgPool::connect(config.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    pool
}

/// Cleans up Postgres from databases created during testing
pub async fn clean_up_database(name: String) {
    let connection = PgPool::connect(
        configuration::get()
            .expect("Failed to get configuration")
            .database
            .connection_string_without_db()
            .expose_secret(),
    )
    .await
    .expect("Failed to connect to Postgres");

    // Disconnect from database before dropping
    connection
        .execute(
            format!(
                r#"select pg_terminate_backend(pid) from pg_stat_activity where datname='{}';"#,
                name
            )
            .as_str(),
        )
        .await
        .expect("Failed to terminate database connection");

    // Drop database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, name).as_str())
        .await
        .expect("Failed to drop database");
}
