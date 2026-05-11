use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub device_id: String,
    pub tank_capacity_litres: f64,
    pub initial_fuel_litres: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub batch_size: usize,
    pub reading_sleep_seconds: u64,

    pub theft_reading_number: i32,
    pub leak_start_reading: i32,
    pub leak_end_reading: i32,
    pub refill_reading_number: i32,

    pub ingestion_url: String,
    pub heartbeat_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string("config.toml")?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
