use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

    pub device_id: Option<uuid::Uuid>,
    pub device_code: Option<String>,
    pub device_status: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct TelemetryHistoryQueryParams {
    pub device_id: uuid::Uuid,

    pub start_time: chrono::DateTime<chrono::Utc>,

    pub end_time: chrono::DateTime<chrono::Utc>,
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

#[derive(Debug, Deserialize)]
pub struct CheckPositionRequest {
    pub organization_id: uuid::Uuid,
    pub device_id: uuid::Uuid,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize)]
pub struct GeofencePositionMatch {
    pub geofence_id: uuid::Uuid,
    pub geofence_name: String,
    pub geofence_type: String,
}

#[derive(Debug, Serialize)]
pub struct CheckPositionResponse {
    pub inside_geofence: bool,
    pub matched_geofences: Vec<GeofencePositionMatch>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GeofenceTransitionEventResponse {
    pub id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub device_id: uuid::Uuid,

    pub geofence_id: uuid::Uuid,
    pub geofence_name: String,
    pub geofence_type: String,

    pub transition_type: String,

    pub latitude: f64,
    pub longitude: f64,

    pub recorded_at: chrono::DateTime<chrono::Utc>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsAlertTrendQuery {
    pub device_id: Option<uuid::Uuid>,
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AlertTrendSummary {
    pub total_alerts: i64,
    pub theft_alerts: i64,
    pub refill_alerts: i64,
    pub leak_alerts: i64,
    pub open_alerts: i64,
    pub acknowledged_alerts: i64,
    pub resolved_alerts: i64,
}

#[derive(Debug, Serialize)]
pub struct AlertTrendPoint {
    pub day: String,
    pub alert_type: String,
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AlertTrendsResponse {
    pub days: i64,
    pub summary: AlertTrendSummary,
    pub trend: Vec<AlertTrendPoint>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsGeofenceActivityQuery {
    pub device_id: Option<uuid::Uuid>,
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GeofenceActivityTrendPoint {
    pub day: String,
    pub entries: i64,
    pub exits: i64,
}

#[derive(Debug, Serialize)]
pub struct GeofenceActivityTrendResponse {
    pub days: i64,
    pub trend: Vec<GeofenceActivityTrendPoint>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsDeviceHealthTrendQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct DeviceHealthTrendDevice {
    pub device_id: uuid::Uuid,
    pub device_code: String,
    pub offline_events: i64,
    pub stale_events: i64,
    pub recovery_events: i64,
    pub reliability_issue_count: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviceHealthTrendResponse {
    pub days: i64,
    pub devices: Vec<DeviceHealthTrendDevice>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsGeofenceUtilizationQuery {
    pub days: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GeofenceUtilizationZone {
    pub geofence_name: String,
    pub visits: i64,
}

#[derive(Debug, Serialize)]
pub struct GeofenceUtilizationResponse {
    pub days: i64,
    pub zones: Vec<GeofenceUtilizationZone>,
}

// -----------------------------------------------------------------------------
// ORBI Device Inventory
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OrbiDeviceInventory {
    pub id: Uuid,

    pub device_code: String,
    pub serial_number: String,
    pub imei: Option<String>,

    pub device_model_id: Uuid,
    pub hardware_profile_id: Uuid,

    pub firmware_version: Option<String>,
    pub production_batch: Option<String>,

    pub inventory_status: String,
    pub quality_test_status: String,

    pub notes: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrbiDeviceInventoryRequest {
    pub device_code: String,
    pub serial_number: String,
    pub imei: Option<String>,

    pub device_model_id: Uuid,
    pub hardware_profile_id: Uuid,

    pub firmware_version: Option<String>,
    pub production_batch: Option<String>,

    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrbiDeviceInventoryStatusRequest {
    pub inventory_status: String,
    pub quality_test_status: String,
}

#[derive(Debug, Serialize)]
pub struct OrbiDeviceInventoryMutationResponse {
    pub inventory_device_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyOrbiDeviceResponse {
    pub found: bool,
    pub device: Option<OrbiDeviceInventory>,
}

// -----------------------------------------------------------------------------
// Device Management
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HardwareProfile {
    pub id: uuid::Uuid,
    pub profile_code: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HardwareProfileSensor {
    pub id: uuid::Uuid,
    pub hardware_profile_id: uuid::Uuid,
    pub sensor_type: String,
    pub unit: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub asset_id: uuid::Uuid,
    pub device_model_id: Option<uuid::Uuid>,
    pub device_code: String,
    pub hardware_profile_id: uuid::Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ProvisionInventoryDeviceRequest {
    pub inventory_device_id: Uuid,
    pub asset_id: Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DeviceSummary {
    pub id: uuid::Uuid,
    pub device_code: String,
    pub asset_id: uuid::Uuid,
    pub device_model_id: Option<uuid::Uuid>,
    pub device_model_code: Option<String>,
    pub device_model_name: Option<String>,
    pub hardware_profile_id: uuid::Uuid,
    pub hardware_profile_code: String,
    pub hardware_profile_name: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DeviceSensorSummary {
    pub id: uuid::Uuid,
    pub device_id: uuid::Uuid,
    pub sensor_code: String,
    pub sensor_type: String,
    pub unit: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub organization_name: String,
    pub industry: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub organization_name: String,
    pub industry: String,
}

#[derive(Debug, Serialize)]
pub struct OrganizationMutationResponse {
    pub organization_id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub organization_id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub name: String,
    pub asset_type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssetMutationResponse {
    pub asset_id: Uuid,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub device_code: String,
    pub hardware_profile_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AssignDeviceAssetRequest {
    pub asset_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct DeviceMutationResponse {
    pub device_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceModelResponse {
    pub id: Uuid,
    pub model_code: String,
    pub model_name: String,
    pub manufacturer: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceCatalogueSensorResponse {
    pub id: Uuid,
    pub sensor_type: String,
    pub unit: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceCatalogueHardwareProfileResponse {
    pub id: Uuid,
    pub profile_code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_default: bool,
    pub sensors: Vec<DeviceCatalogueSensorResponse>,
}

#[derive(Debug, Serialize)]
pub struct DeviceCatalogueModelResponse {
    pub id: Uuid,
    pub model_code: String,
    pub model_name: String,
    pub manufacturer: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub profiles: Vec<DeviceCatalogueHardwareProfileResponse>,
}
