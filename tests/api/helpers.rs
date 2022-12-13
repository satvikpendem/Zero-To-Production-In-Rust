use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Executor, PgPool};
use std::io::{sink, stdout};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    startup::{get_connection_pool, Application},
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

    let configuration = {
        let mut c = configuration::get().expect("Failed to read configuration");
        // Use a different database for each test case
        c.database.name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    let application = Application::build(&configuration)
        .await
        .expect("Failed to build application");

    // Get address before spawning the application
    let address = format!("http://127.0.0.1:{}", application.port());
    tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        database_pool: get_connection_pool(&configuration.database),
        database_name: configuration.database.name,
    }
}

async fn configure_database(configuration: &DatabaseSettings) -> PgPool {
    let connection = PgPool::connect(configuration.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, configuration.name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(configuration.connection_string().expose_secret())
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
