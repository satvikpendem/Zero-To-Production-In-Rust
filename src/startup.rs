use std::net::TcpListener;

use actix_web::{
    dev::Server,
    web::{self, Data},
    App, HttpServer,
};
use sqlx::PgPool;

use crate::routes::{health::health, subscriptions::subscribe};

pub fn run(listener: TcpListener, connection_pool: PgPool) -> std::io::Result<Server> {
    let pool = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(pool.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
