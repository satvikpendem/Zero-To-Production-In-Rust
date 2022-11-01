use std::net::TcpListener;

use sqlx::{Executor, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{self, DatabaseSettings},
    startup::run,
};

#[derive(Clone)]
pub struct TestApp {
    pub address: String,
    pub database_pool: PgPool,
    pub database_name: String,
}

async fn spawn_app() -> TestApp {
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
    let connection = PgPool::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    pool
}

pub async fn clean_up_database(name: String) {
    let connection = PgPool::connect(
        &configuration::get()
            .expect("Failed to get configuration")
            .database
            .connection_string_without_db(),
    )
    .await
    .expect("Failed to connect to Postgres");

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

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    clean_up_database(database_name).await;
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
        .post(url)
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

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    clean_up_database(database_name).await;
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let TestApp {
        address,
        database_pool: _,
        database_name,
    } = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing name and email"),
    ];

    let url = format!("{address}/subscriptions");

    for (body, error_message) in test_cases {
        let response = client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with `400 Bad Request` when the payload was `{error_message}`"
        );
    }

    clean_up_database(database_name).await;
}
