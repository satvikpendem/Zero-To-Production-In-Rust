use crate::helpers::{clean_up_database, spawn_app, TestApp};

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
