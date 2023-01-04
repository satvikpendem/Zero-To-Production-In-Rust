use linkify::{LinkFinder, LinkKind};
use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{clean_up_database, spawn_app};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_400() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();
    clean_up_database(app.database_name).await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_200_if_called() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    // Extract the link from one of the request fields.
    let get_link = |s: &str| {
        let links: Vec<_> = LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = &get_link(body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.port)).unwrap();

    // Act
    let response = reqwest::get(confirmation_link).await.unwrap();
    clean_up_database(app.database_name).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}