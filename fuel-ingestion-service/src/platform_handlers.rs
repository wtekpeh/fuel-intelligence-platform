use crate::repository;
use crate::routes::AppState;
use axum::extract::Path;
use axum::{Json, extract::State};
use uuid::Uuid;

pub async fn list_hardware_profiles_handler(
    State(app_state): State<AppState>,
) -> Json<Vec<crate::models::HardwareProfile>> {
    let profiles = repository::list_hardware_profiles(&app_state.db_pool)
        .await
        .expect("Failed to list hardware profiles");

    Json(profiles)
}

pub async fn list_hardware_profile_sensors_handler(
    State(app_state): State<AppState>,
    Path(hardware_profile_id): Path<Uuid>,
) -> Json<Vec<crate::models::HardwareProfileSensor>> {
    let sensors = repository::get_hardware_profile_sensors(&app_state.db_pool, hardware_profile_id)
        .await
        .expect("Failed to list hardware profile sensors");

    Json(sensors)
}

pub async fn register_device_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::RegisterDeviceRequest>,
) -> Json<uuid::Uuid> {
    let device_id = repository::register_device(
        &app_state.db_pool,
        payload.asset_id,
        payload.device_code,
        payload.hardware_profile_id,
    )
    .await
    .expect("Failed to register device");

    Json(device_id)
}

pub async fn list_devices_handler(
    State(app_state): State<AppState>,
) -> Json<Vec<crate::models::DeviceSummary>> {
    let devices = repository::list_devices(&app_state.db_pool)
        .await
        .expect("Failed to list devices");

    Json(devices)
}

pub async fn list_device_sensors_handler(
    State(app_state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Json<Vec<crate::models::DeviceSensorSummary>> {
    let sensors = repository::list_device_sensors(&app_state.db_pool, device_id)
        .await
        .expect("Failed to list device sensors");

    Json(sensors)
}
