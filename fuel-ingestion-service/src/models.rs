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

#[derive(Debug, Serialize)]
pub struct FuelEventResponse {
    pub id: String,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    pub detected_at: DateTime<Utc>,
    pub fuel_before: f64,
    pub fuel_after: f64,
    pub fuel_difference: f64,
    pub duration_seconds: i64,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_delayed_detection: bool,
    pub sync_delay_seconds: i64,
    pub severity: String,
    pub message: String,
}
