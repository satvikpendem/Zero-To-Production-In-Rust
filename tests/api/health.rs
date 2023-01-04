use crate::helpers::{clean_up_database, spawn_app, TestApp};

#[tokio::test]
async fn health() {
    let TestApp {
        address,
        database_pool: _,
        database_name,
        email_server: _,
        port: _,
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
