use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use sqlx::PgPool;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration,
    email_client::EmailClient,
    routes::{health::health, subscriptions::subscribe},
};

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let pool = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    let config = configuration::get().expect("Failed to read configuration.");
    info!(
        "Starting on {}:{}",
        config.application.host, config.application.port,
    );

    Ok(server)
}
