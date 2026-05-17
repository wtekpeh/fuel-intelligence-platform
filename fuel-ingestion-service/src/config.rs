use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub device_stale_after_seconds: i64,
    pub device_offline_after_seconds: i64,
    pub device_health_refresh_interval_seconds: u64,
    pub default_tank_capacity_litres: f64,
    pub max_allowed_fuel_jump_litres: f64,
    pub fuel_rolling_window_size: usize,
    pub fuel_iqr_multiplier: f64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("SERVER_PORT must be a valid number")?;

        let device_stale_after_seconds = env::var("DEVICE_STALE_AFTER_SECONDS")
            .unwrap_or_else(|_| "120".to_string())
            .parse::<i64>()
            .context("DEVICE_STALE_AFTER_SECONDS must be a valid number")?;

        let device_offline_after_seconds = env::var("DEVICE_OFFLINE_AFTER_SECONDS")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<i64>()
            .context("DEVICE_OFFLINE_AFTER_SECONDS must be a valid number")?;

        let device_health_refresh_interval_seconds =
            env::var("DEVICE_HEALTH_REFRESH_INTERVAL_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse::<u64>()
                .context("DEVICE_HEALTH_REFRESH_INTERVAL_SECONDS must be a valid number")?;
        let default_tank_capacity_litres = env::var("DEFAULT_TANK_CAPACITY_LITRES")
            .unwrap_or_else(|_| "200".to_string())
            .parse::<f64>()
            .context("DEFAULT_TANK_CAPACITY_LITRES must be a valid number")?;

        let max_allowed_fuel_jump_litres = env::var("MAX_ALLOWED_FUEL_JUMP_LITRES")
            .unwrap_or_else(|_| "80".to_string())
            .parse::<f64>()
            .context("MAX_ALLOWED_FUEL_JUMP_LITRES must be a valid number")?;

        let fuel_rolling_window_size = env::var("FUEL_ROLLING_WINDOW_SIZE")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<usize>()
            .context("FUEL_ROLLING_WINDOW_SIZE must be a valid number")?;

        let fuel_iqr_multiplier = env::var("FUEL_IQR_MULTIPLIER")
            .unwrap_or_else(|_| "1.5".to_string())
            .parse::<f64>()
            .context("FUEL_IQR_MULTIPLIER must be a valid number")?;

        Ok(Self {
            database_url,
            server_host,
            server_port,
            device_stale_after_seconds,
            device_offline_after_seconds,
            device_health_refresh_interval_seconds,
            default_tank_capacity_litres,
            max_allowed_fuel_jump_litres,
            fuel_rolling_window_size,
            fuel_iqr_multiplier,
        })
    }
}
