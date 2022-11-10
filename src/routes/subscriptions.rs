use actix_web::{
    web::{Data, Form},
    HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Subscribe someone to the newsletter
/// # Panics
/// When the database cannot be found or connected to
#[tracing::instrument(
    name = "Adding new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: Form<FormData>, pool: Data<PgPool>) -> HttpResponse {
    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let subscriber = NewSubscriber {
        name,
        email: form.0.email,
    };

    // insert_subscriber(&subscriber, &pool)
    //     .await
    //     .map_err(|_| HttpResponse::InternalServerError().finish())?;

    match insert_subscriber(&subscriber, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(subscriber, pool)
)]
pub async fn insert_subscriber(
    subscriber: &NewSubscriber,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        subscriber.email,
        subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
