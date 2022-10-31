use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::{configuration, startup::run};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = configuration::get().expect("Unable to read configuration");
    let address = TcpListener::bind("127.0.0.1:8000")?;
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    run(address, connection_pool)?.await
}
