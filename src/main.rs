use anyhow::{Context, Result};
use dotenv::dotenv;
use log::{debug, trace};

pub mod utils;

mod config;
mod routes;
mod services;

#[rocket::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cfg = config::new()?;

    env_logger::builder()
        .filter_level(cfg.debug.log)
        .format_target(false)
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

    let rocket = rocket::custom(cfg.rocket())
        .attach(routes::attach())
        .attach(services::attach())
        .manage(cfg)
        .manage(pool)
        .launch()
        .await?;
    debug!("{rocket:?}");
    Ok(())
}
