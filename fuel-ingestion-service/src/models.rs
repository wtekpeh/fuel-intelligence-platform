use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FuelReading {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub fuel_level_litres: f64,
    pub fuel_level_percentage: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub simulation_mode: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReadingBatch {
    pub device_id: String,
    pub synced_at: DateTime<Utc>,
    pub readings: Vec<FuelReading>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
    pub received_count: usize,
}
