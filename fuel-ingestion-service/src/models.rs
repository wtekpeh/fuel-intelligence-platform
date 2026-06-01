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

    pub vibration_level: f64,
    pub motion_detected: bool,

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
    pub confidence: Option<String>,
    pub correlation_status: Option<String>,
    pub correlation_reason: Option<String>,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HeartbeatRequest {
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub success: bool,
    pub message: String,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceHealthEventResponse {
    pub id: String,
    pub device_id: String,
    pub previous_status: Option<String>,
    pub new_status: String,
    pub reason: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct SensorHealthEventResponse {
    pub id: String,
    pub device_id: String,
    pub sensor_id: String,
    pub event_type: String,
    pub severity: String,
    pub reason: String,
    pub first_seen_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DeviceStateEventResponse {
    pub state: String,

    pub vibration_level: Option<f64>,
    pub motion_detected: Option<bool>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertResponse {
    pub id: uuid::Uuid,
    pub fuel_event_id: Option<uuid::Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub reason: String,
    pub is_acknowledged: bool,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub device_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertAcknowledgementResponse {
    pub id: uuid::Uuid,
    pub alert_type: String,
    pub severity: String,
    pub is_acknowledged: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TelemetryStreamResponse {
    pub device_id: String,

    pub fuel_level_litres: f64,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub vibration_level: Option<f64>,
    pub motion_detected: Option<bool>,

    pub recorded_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct OrganizationOverviewResponse {
    pub organization_id: uuid::Uuid,

    pub organization_name: String,

    pub industry: Option<String>,

    pub asset_count: i64,

    pub device_count: i64,

    pub online_device_count: i64,

    pub stale_device_count: i64,

    pub offline_device_count: i64,

    pub open_alert_count: i64,
}

#[derive(Debug, Serialize)]
pub struct OrganizationFleetOverviewResponse {
    pub asset_id: uuid::Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub capacity_litres: Option<f64>,

    pub device_id: uuid::Uuid,
    pub device_code: String,
    pub device_status: String,
    pub last_seen_at: Option<DateTime<Utc>>,

    pub sensor_count: i64,
    pub sensor_types: Vec<String>,

    pub open_alert_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    pub device_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct TelemetryQueryParams {
    pub device_id: Option<uuid::Uuid>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Geofence {
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub name: String,
    pub geofence_type: String,
    pub geojson: serde_json::Value,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateGeofenceRequest {
    pub organization_id: uuid::Uuid,
    pub name: String,
    pub geofence_type: String,
    pub geojson: serde_json::Value,
}
