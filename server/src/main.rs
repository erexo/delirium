use anyhow::{Context, Result};
use dotenv::dotenv;
use poem::{listener::TcpListener, Server};
use tracing::trace;

pub mod utils;

mod api;
mod config;
mod services;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = config::get();

    tracing_subscriber::fmt()
        .with_max_level(cfg.debug.log)
        .with_target(false)
        .init();

    trace!("hi");

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(cfg.database.connections)
        .connect(&format!(
            "mysql://{}:{}@{}/{}",
            cfg.database.user, cfg.database.password, cfg.database.host, cfg.database.database
        ))
        .await
        .context("database connection")?;
    let jwt = services::jwt::new();

    Server::new(TcpListener::bind((cfg.api.address, cfg.api.port)))
        .run(api::routes(&pool, jwt))
        .await
        .context("server start")
}
