use crate::repository;
use crate::routes::AppState;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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

pub async fn list_device_models_handler(
    State(app_state): State<AppState>,
) -> Json<Vec<crate::models::DeviceModelResponse>> {
    let models = repository::list_device_models(&app_state.db_pool)
        .await
        .expect("Failed to list device models");

    Json(models)
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
) -> impl IntoResponse {
    match repository::register_device(
        &app_state.db_pool,
        payload.asset_id,
        payload.device_model_id,
        payload.device_code,
        payload.hardware_profile_id,
    )
    .await
    {
        Ok(device_id) => (StatusCode::CREATED, Json(device_id)).into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("devices_device_code_key") {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "Device code already exists.".to_string(),
                    }),
                )
                    .into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to register device.".to_string(),
                }),
            )
                .into_response()
        }
    }
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

pub async fn create_organization_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::CreateOrganizationRequest>,
) -> (
    StatusCode,
    Json<crate::models::OrganizationMutationResponse>,
) {
    let organization_id = repository::create_organization(&app_state.db_pool, &payload)
        .await
        .expect("Failed to create organization");

    (
        StatusCode::CREATED,
        Json(crate::models::OrganizationMutationResponse {
            organization_id,
            message: "Organization created successfully.".to_string(),
        }),
    )
}

pub async fn update_organization_handler(
    State(app_state): State<AppState>,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateOrganizationRequest>,
) -> Json<crate::models::OrganizationMutationResponse> {
    repository::update_organization(&app_state.db_pool, organization_id, &payload)
        .await
        .expect("Failed to update organization");

    Json(crate::models::OrganizationMutationResponse {
        organization_id,
        message: "Organization updated successfully.".to_string(),
    })
}

pub async fn delete_organization_handler(
    State(app_state): State<AppState>,
    Path(organization_id): Path<Uuid>,
) -> Json<crate::models::OrganizationMutationResponse> {
    repository::archive_organization(&app_state.db_pool, organization_id)
        .await
        .expect("Failed to delete organization");

    Json(crate::models::OrganizationMutationResponse {
        organization_id,
        message: "Organization deactivated successfully.".to_string(),
    })
}

pub async fn create_asset_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::CreateAssetRequest>,
) -> (StatusCode, Json<crate::models::AssetMutationResponse>) {
    let asset_id = repository::create_asset(&app_state.db_pool, &payload)
        .await
        .expect("Failed to create asset");

    (
        StatusCode::CREATED,
        Json(crate::models::AssetMutationResponse {
            asset_id,
            message: "Asset created successfully.".to_string(),
        }),
    )
}

pub async fn update_asset_handler(
    State(app_state): State<AppState>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateAssetRequest>,
) -> Json<crate::models::AssetMutationResponse> {
    repository::update_asset(&app_state.db_pool, asset_id, &payload)
        .await
        .expect("Failed to update asset");

    Json(crate::models::AssetMutationResponse {
        asset_id,
        message: "Asset updated successfully.".to_string(),
    })
}

pub async fn delete_asset_handler(
    State(app_state): State<AppState>,
    Path(asset_id): Path<Uuid>,
) -> Json<crate::models::AssetMutationResponse> {
    repository::archive_asset(&app_state.db_pool, asset_id)
        .await
        .expect("Failed to deactivate asset");

    Json(crate::models::AssetMutationResponse {
        asset_id,
        message: "Asset deactivated successfully.".to_string(),
    })
}

pub async fn update_device_handler(
    State(app_state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateDeviceRequest>,
) -> Json<crate::models::DeviceMutationResponse> {
    repository::update_device(&app_state.db_pool, device_id, &payload)
        .await
        .expect("Failed to update device");

    Json(crate::models::DeviceMutationResponse {
        device_id,
        message: "Device updated successfully.".to_string(),
    })
}

pub async fn delete_device_handler(
    State(app_state): State<AppState>,
    Path(device_id): Path<Uuid>,
) -> Json<crate::models::DeviceMutationResponse> {
    repository::deactivate_device(&app_state.db_pool, device_id)
        .await
        .expect("Failed to deactivate device");

    Json(crate::models::DeviceMutationResponse {
        device_id,
        message: "Device deactivated successfully.".to_string(),
    })
}

pub async fn assign_device_asset_handler(
    State(app_state): State<AppState>,
    Path(device_id): Path<Uuid>,
    Json(payload): Json<crate::models::AssignDeviceAssetRequest>,
) -> Json<crate::models::DeviceMutationResponse> {
    repository::assign_device_to_asset(&app_state.db_pool, device_id, &payload)
        .await
        .expect("Failed to assign device");

    Json(crate::models::DeviceMutationResponse {
        device_id,
        message: "Device assigned successfully.".to_string(),
    })
}
