use actix_web::{
    web::{Data, Form},
    HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{query, PgPool};
use tracing::{error, info, info_span, Instrument};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Subscribe someone to the newsletter
/// # Panics
/// When the database cannot be found or connected to
pub async fn subscribe(form: Form<FormData>, connection: Data<PgPool>) -> HttpResponse {
    let FormData { name, email } = form.0;
    let request_id = Uuid::new_v4();

    let request_span = info_span!("Adding new subscriber", %request_id, subscriber_email = %email, subscriber_name = name);
    let _request_span_guard = request_span.enter();

    let query_span = info_span!("Saving new subscriber details in the database");

    if let Err(e) = query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        email,
        name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .instrument(query_span)
    .await
    {
        error!("{request_id} - Failed to execute query {:?}", e);
        HttpResponse::InternalServerError().body("Email already exists")
    } else {
        info!("{request_id} - New subscriber details have been saved");
        HttpResponse::Ok().finish()
    }
}
