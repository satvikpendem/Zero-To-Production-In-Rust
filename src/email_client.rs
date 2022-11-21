use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
// use serde::{Deserialize, Serialize};

use crate::domain::SubscriberEmail;
use crate::sendgrid_email_format::{
    ContentField, FromField, PersonalizationField, SendgridEmailFormat, ToField,
};

#[derive(Clone)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    authorization_token: Secret<String>,
}

// #[derive(Serialize)]
// struct SendEmailRequest {
//     from: String,
//     to: String,
//     subject: String,
//     content: String,
// }

impl EmailClient {
    #[must_use]
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendgridEmailFormat {
            personalizations: vec![PersonalizationField {
                to: vec![ToField {
                    email: recipient.as_ref().to_string(),
                }],
            }],
            from: FromField {
                email: "satvik@darkmodes.com".to_string(),
            },
            subject: subject.to_string(),
            content: vec![ContentField {
                type_field: "text/plain".to_string(),
                value: text_content.to_string(),
            }],
        };
        let builder = self
            .http_client
            .post(&url)
            .bearer_auth(self.authorization_token.expose_secret())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{matchers::header_exists, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client =
            EmailClient::new(mock_server.uri(), sender_email, Secret::new(Faker.fake()));

        Mock::given(header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
