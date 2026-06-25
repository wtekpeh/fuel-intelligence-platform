mod config;
mod db;
mod handlers;
mod models;
mod platform_handlers;
mod platform_routes;
mod repository;
mod routes;
mod services;
mod ws;

use crate::services::alert_hub::AlertHub;
use config::AppConfig;
use db::create_db_pool;
use repository::refresh_device_statuses;
use routes::app_routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;

    let db_pool = create_db_pool(&config.database_url).await?;

    let health_db_pool = db_pool.clone();

    let stale_after_seconds = config.device_stale_after_seconds;
    let offline_after_seconds = config.device_offline_after_seconds;
    let health_refresh_interval_seconds = config.device_health_refresh_interval_seconds;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            health_refresh_interval_seconds,
        ));

        loop {
            interval.tick().await;

            match refresh_device_statuses(
                &health_db_pool,
                stale_after_seconds,
                offline_after_seconds,
            )
            .await
            {
                Ok(_) => {
                    println!("Device health refresh completed.");
                }

                Err(err) => {
                    eprintln!("Automatic device health refresh failed: {}", err);
                }
            }
        }
    });

    let alert_hub = AlertHub::new();

    let app = app_routes(db_pool, config.clone(), alert_hub);

    let address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&address).await?;

    println!("Fuel ingestion service running on http://{}", address);

    axum::serve(listener, app).await?;

    Ok(())
}
