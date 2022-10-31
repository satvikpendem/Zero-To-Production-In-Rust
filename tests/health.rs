use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration, startup::run};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let configuration = configuration::get().expect("Failed to read configuration.");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp { address, db_pool }
}

#[tokio::test]
async fn health() {
    let TestApp { address, db_pool } = spawn_app().await;
    let client = reqwest::Client::new();

    let url = format!("{address}/health");

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let TestApp { address, db_pool } = spawn_app().await;

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
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let TestApp { address, db_pool } = spawn_app().await;
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
}
