use std::{
    io::{sink, stdout},
    net::TcpListener,
};

use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
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

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut config = configuration::get().expect("Failed to read configuration.");
    config.database.name = Uuid::new_v4().to_string();
    let database_pool = configure_database(&config.database).await;

    let server = run(listener, database_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp {
        address,
        database_pool,
        database_name: config.database.name,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
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

#[tokio::test]
async fn health() {
    let TestApp {
        address,
        database_pool: _,
        database_name,
    } = spawn_app().await;

    let client = reqwest::Client::new();

    let url = format!("{address}/health");

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request.");

    // Clean up database *first* before `assert`ing which might cause a panic
    clean_up_database(database_name).await;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let TestApp {
        address,
        database_pool,
        database_name,
    } = spawn_app().await;

    let client = reqwest::Client::new();
    let url = format!("{address}/subscriptions");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&database_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    clean_up_database(database_name).await;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing name and email"),
    ];

    for (body, error_message) in test_cases {
        let TestApp {
            address,
            database_pool: _,
            database_name,
        } = spawn_app().await;

        let url = format!("{address}/subscriptions");
        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        clean_up_database(database_name).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with `400 Bad Request` when the payload was `{error_message}`"
        );
    }
}

#[tokio::test]
async fn subscribe_returns_500_when_email_already_exists() {
    let TestApp {
        address,
        database_pool: _,
        database_name,
    } = spawn_app().await;

    let client = reqwest::Client::new();
    let url = format!("{address}/subscriptions");

    // Send first request with form data
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    // Send second request with duplicate form data
    let response = client
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    clean_up_database(database_name).await;

    assert_eq!(500, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
