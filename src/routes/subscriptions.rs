use actix_web::{
    web::{Data, Form},
    HttpResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{query, PgPool};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: Form<FormData>, connection: Data<PgPool>) -> HttpResponse {
    if let Err(e) = query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        println!("Failed to execute query {e}");
        HttpResponse::InternalServerError().finish()
    } else {
        HttpResponse::Ok().finish()
    }
}
