use crate::catalogue_repository;
use crate::domain::operational_behaviour::BehaviourType;
use crate::operational_behaviour_repository;
use crate::orbi_inventory_repository;
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

pub async fn list_device_catalogue_handler(
    State(app_state): State<AppState>,
) -> Json<Vec<crate::models::DeviceCatalogueModelResponse>> {
    let catalogue = catalogue_repository::list_device_catalogue(&app_state.db_pool)
        .await
        .expect("Failed to load device catalogue");

    Json(catalogue)
}

//--------Inventory Management
pub async fn create_orbi_inventory_device_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::CreateOrbiDeviceInventoryRequest>,
) -> impl IntoResponse {
    match orbi_inventory_repository::create_orbi_inventory_device(&app_state.db_pool, &payload)
        .await
    {
        Ok(inventory_device_id) => (
            StatusCode::CREATED,
            Json(crate::models::OrbiDeviceInventoryMutationResponse {
                inventory_device_id,
                message: "ORBI device added to inventory successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("orbi_device_inventory_device_code_key") {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "Device code already exists.".to_string(),
                    }),
                )
                    .into_response();
            }

            if message.contains("orbi_device_inventory_serial_number_key") {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "Serial number already exists.".to_string(),
                    }),
                )
                    .into_response();
            }

            if message.contains("orbi_device_inventory_imei_key") {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "IMEI already exists.".to_string(),
                    }),
                )
                    .into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create ORBI inventory device.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn list_orbi_inventory_devices_handler(
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    match orbi_inventory_repository::list_orbi_inventory_devices(&app_state.db_pool).await {
        Ok(devices) => (StatusCode::OK, Json(devices)).into_response(),

        Err(error) => {
            eprintln!("Failed to list ORBI inventory devices: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to list ORBI inventory devices.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn verify_orbi_inventory_device_handler(
    State(app_state): State<AppState>,
    Path(device_code): Path<String>,
) -> impl IntoResponse {
    match orbi_inventory_repository::verify_orbi_inventory_device_by_code(
        &app_state.db_pool,
        &device_code,
    )
    .await
    {
        Ok(device) => (
            StatusCode::OK,
            Json(crate::models::VerifyOrbiDeviceResponse {
                found: device.is_some(),
                device,
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to verify ORBI inventory device: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to verify ORBI inventory device.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn update_orbi_inventory_status_handler(
    State(app_state): State<AppState>,
    Path(inventory_device_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateOrbiDeviceInventoryStatusRequest>,
) -> impl IntoResponse {
    match crate::services::inventory_lifecycle::update_inventory_status(
        &app_state.db_pool,
        inventory_device_id,
        &payload.inventory_status,
        &payload.quality_test_status,
    )
    .await
    {
        Ok(_) => Json(crate::models::OrbiDeviceInventoryMutationResponse {
            inventory_device_id,
            message: "ORBI inventory status updated successfully.".to_string(),
        })
        .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("Invalid inventory lifecycle transition")
                || message.contains("Unknown current inventory status")
                || message.contains("Unknown requested inventory status")
                || message.contains("Inventory device not found")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!("Failed to update ORBI inventory status: {}", message);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to update ORBI inventory status.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn get_orbi_inventory_device_handler(
    State(app_state): State<AppState>,
    Path(inventory_device_id): Path<Uuid>,
) -> impl IntoResponse {
    match orbi_inventory_repository::get_orbi_inventory_device(
        &app_state.db_pool,
        inventory_device_id,
    )
    .await
    {
        Ok(Some(device)) => (StatusCode::OK, Json(device)).into_response(),

        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(crate::models::ApiErrorResponse {
                message: "ORBI inventory device not found.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to fetch ORBI inventory device: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to fetch ORBI inventory device.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn provision_inventory_device_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::ProvisionInventoryDeviceRequest>,
) -> impl IntoResponse {
    match repository::provision_inventory_device(&app_state.db_pool, &payload).await {
        Ok(device_id) => (
            StatusCode::CREATED,
            Json(crate::models::DeviceMutationResponse {
                device_id,
                message: "Inventory device provisioned successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("Inventory device not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("already been provisioned")
                || message.contains("Retired inventory device")
                || message.contains("Device code already exists")
            {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to provision inventory device.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

//--------Guided Fuel Calibration Management

pub async fn create_fuel_calibration_profile_handler(
    State(app_state): State<AppState>,
    Path(sensor_id): Path<Uuid>,
    Json(payload): Json<crate::models::CreateFuelCalibrationProfileRequest>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::create_profile(
        &app_state.db_pool,
        sensor_id,
        payload.tank_capacity_litres,
    )
    .await
    {
        Ok(profile_id) => (
            StatusCode::CREATED,
            Json(crate::models::FuelCalibrationProfileMutationResponse {
                profile_id,
                message: "Fuel calibration profile created successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            /*
             * A nonexistent sensor is a resource lookup failure.
             */
            if message.contains("Sensor not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            /*
             * The guided fuel-calibration workflow may only be attached
             * to an installed FUEL sensor.
             */
            if message.contains("not a FUEL sensor") || message.contains("Tank capacity") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            /*
             * A sensor must not have two simultaneous current calibration
             * profiles. Historical superseded profiles are allowed, but
             * only one current workflow may exist.
             */
            if message.contains("already has a current fuel calibration profile")
                || message.contains("unique_current_fuel_calibration_profile")
            {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to create guided fuel calibration profile for sensor {}: {}",
                sensor_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create fuel calibration profile.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn start_fuel_calibration_session_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<Uuid>,
    Json(payload): Json<crate::models::StartFuelCalibrationSessionRequest>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::start_session(
        &app_state.db_pool,
        profile_id,
        payload.starting_litres,
    )
    .await
    {
        Ok(session_id) => (
            StatusCode::CREATED,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration session started successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("profile not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Starting fuel quantity") || message.contains("superseded profile")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("already has an unfinished session")
                || message.contains("unique_unfinished_fuel_calibration_session")
            {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to start guided fuel calibration session for profile {}: {}",
                profile_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to start fuel calibration session.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

//--------Sensor Calibration Management

pub async fn create_sensor_calibration_handler(
    State(app_state): State<AppState>,
    Path(sensor_id): Path<Uuid>,
    Json(payload): Json<crate::models::CreateSensorCalibrationRequest>,
) -> impl IntoResponse {
    match crate::services::platform::calibration::create_calibration(
        &app_state.db_pool,
        sensor_id,
        payload,
    )
    .await
    {
        Ok(calibration_id) => (
            StatusCode::CREATED,
            Json(crate::models::SensorCalibrationMutationResponse {
                calibration_id,
                message: "Sensor calibration created successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("Sensor not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Calibration type") || message.contains("Calibration values") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!("Failed to create sensor calibration: {}", message);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create sensor calibration.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn list_sensor_calibrations_handler(
    State(app_state): State<AppState>,
    Path(sensor_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::calibration::list_calibration_history(
        &app_state.db_pool,
        sensor_id,
    )
    .await
    {
        Ok(calibrations) => (StatusCode::OK, Json(calibrations)).into_response(),

        Err(error) => {
            eprintln!("Failed to list sensor calibrations: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to list sensor calibrations.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn get_active_sensor_calibration_handler(
    State(app_state): State<AppState>,
    Path((sensor_id, calibration_type)): Path<(Uuid, String)>,
) -> impl IntoResponse {
    match crate::services::platform::calibration::get_active_calibration(
        &app_state.db_pool,
        sensor_id,
        &calibration_type,
    )
    .await
    {
        Ok(Some(calibration)) => (StatusCode::OK, Json(calibration)).into_response(),

        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(crate::models::ApiErrorResponse {
                message: format!(
                    "No active {} calibration was found for this sensor.",
                    calibration_type.trim().to_uppercase()
                ),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("Calibration type") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!("Failed to fetch active sensor calibration: {}", message);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to fetch active sensor calibration.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn create_operational_behaviour_learning_session_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::CreateOperationalBehaviourLearningSessionRequest>,
) -> impl IntoResponse {
    let behaviour_type = match BehaviourType::from_str(&payload.behaviour_type) {
        Some(value) => value,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(crate::models::ApiErrorResponse {
                    message: format!(
                        "Unknown behaviour type '{}'. Expected PARKED, IDLE or MOVING.",
                        payload.behaviour_type
                    ),
                }),
            )
                .into_response();
        }
    };

    match operational_behaviour_repository::create_learning_session(
        &app_state.db_pool,
        payload.device_id,
        payload.sensor_id,
        behaviour_type,
        payload.requested_sample_count,
    )
    .await
    {
        Ok(learning_session) => (
            StatusCode::CREATED,
            Json(
                crate::models::OperationalBehaviourLearningSessionMutationResponse {
                    learning_session_id: learning_session.id,
                    message: "Operational behaviour learning session created successfully."
                        .to_string(),
                },
            ),
        )
            .into_response(),

        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(crate::models::ApiErrorResponse {
                message: error.to_string(),
            }),
        )
            .into_response(),
    }
}

pub async fn start_operational_behaviour_learning_session_handler(
    State(app_state): State<AppState>,
    Path(learning_session_id): Path<Uuid>,
) -> impl IntoResponse {
    match operational_behaviour_repository::start_learning_session(
        &app_state.db_pool,
        learning_session_id,
    )
    .await
    {
        Ok(Some(session)) => (
            StatusCode::OK,
            Json(
                crate::models::OperationalBehaviourLearningSessionMutationResponse {
                    learning_session_id: session.id,
                    message: "Operational behaviour learning session started successfully."
                        .to_string(),
                },
            ),
        )
            .into_response(),

        Ok(None) => (
            StatusCode::CONFLICT,
            Json(crate::models::ApiErrorResponse {
                message: "Learning session was not found or is not in NOT_STARTED status."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!(
                "Failed to start operational behaviour learning session {}: {}",
                learning_session_id, error
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to start operational behaviour learning session.".to_string(),
                }),
            )
                .into_response()
        }
    }
}
