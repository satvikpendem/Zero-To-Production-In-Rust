use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self, Form},
    App, HttpResponse, HttpServer,
};
use serde::Deserialize;

pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(_form: Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
