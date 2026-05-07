mod config;
mod db;
mod handlers;
mod models;
mod repository;
mod routes;

use config::AppConfig;
use db::create_db_pool;
use routes::app_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;

    let db_pool = create_db_pool(&config.database_url).await?;

    let app = app_routes(db_pool);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("Fuel ingestion service running on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}
