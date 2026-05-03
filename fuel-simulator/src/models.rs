use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct FuelReading {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub fuel_level_litres: f64,
    pub fuel_level_percentage: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub simulation_mode: String,
}

#[derive(Debug, Clone, Copy)]
pub enum SimulationMode {
    Normal,
    Theft,
    Leak,
    Refill,
}

#[derive(Debug, Serialize)]
pub struct ReadingBatch {
    pub device_id: String,
    pub synced_at: DateTime<Utc>,
    pub readings: Vec<FuelReading>,
}
