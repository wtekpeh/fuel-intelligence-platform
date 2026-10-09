use crate::auth::claims::KeycloakClaims;
use crate::catalogue_repository;
use crate::device_activation_repository;
use crate::domain::operational_behaviour::BehaviourType;
use crate::models::OrbiUser;
use crate::operational_behaviour_repository;
use crate::orbi_inventory_repository;
use crate::repository;
use crate::routes::AppState;

use axum::Extension;
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

/// Complete self-service onboarding for a verified Keycloak identity.
///
/// Creates an ORBI user, organization, and initial ADMIN membership
/// through a single PostgreSQL transaction.
pub async fn create_client_onboarding_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<KeycloakClaims>,
    Json(payload): Json<crate::models::ClientOnboardingRequest>,
) -> impl IntoResponse {
    let organization_name = payload.organization_name.trim();
    let industry = payload.industry.trim();

    // Validate organization information.
    if organization_name.is_empty() || industry.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(crate::models::ApiErrorResponse {
                message: "Organization name and industry are required.".to_string(),
            }),
        )
            .into_response();
    }

    if organization_name.len() > 200 || industry.len() > 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(crate::models::ApiErrorResponse {
                message: "Organization information exceeds allowed length.".to_string(),
            }),
        )
            .into_response();
    }

    let result = repository::create_client_onboarding(
        &app_state.db_pool,
        &claims.sub,
        claims.preferred_username.as_deref(),
        claims.email.as_deref(),
        organization_name,
        industry,
    )
    .await;

    match result {
        Ok((user_id, organization_id)) => (
            StatusCode::CREATED,
            Json(crate::models::ClientOnboardingResponse {
                user_id,
                organization_id,
                message: "Client onboarding completed successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            // The unique Keycloak subject prevents an existing ORBI
            // identity from registering another organization.
            //
            // This also prevents disabled accounts from re-registering.
            let duplicate_identity = error
                .downcast_ref::<sqlx::Error>()
                .and_then(|sqlx_error| match sqlx_error {
                    sqlx::Error::Database(database_error) => {
                        Some(database_error.constraint() == Some("orbi_users_keycloak_subject_key"))
                    }
                    _ => None,
                })
                .unwrap_or(false);

            if duplicate_identity {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "This account has already been registered with ORBI.".to_string(),
                    }),
                )
                    .into_response();
            }

            eprintln!("Client onboarding failed: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Unable to complete client onboarding.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Determine whether a verified Keycloak identity has completed
/// ORBI customer onboarding.
pub async fn get_client_onboarding_status_handler(
    State(app_state): State<AppState>,
    Extension(claims): Extension<KeycloakClaims>,
) -> impl IntoResponse {
    let user = match repository::find_orbi_user_by_keycloak_subject(&app_state.db_pool, &claims.sub)
        .await
    {
        Ok(user) => user,

        Err(error) => {
            eprintln!("Failed to resolve onboarding status: {error}");

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let Some(user) = user else {
        return (
            StatusCode::OK,
            Json(crate::models::ClientOnboardingStatusResponse {
                status: "onboarding_required".to_string(),
                onboarding_complete: false,
                user_id: None,
            }),
        )
            .into_response();
    };

    if !user.is_active {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Internal platform identities are not customer onboarding accounts.
    if user.platform_role.is_some() {
        return (
            StatusCode::OK,
            Json(crate::models::ClientOnboardingStatusResponse {
                status: "platform_user".to_string(),
                onboarding_complete: true,
                user_id: Some(user.id),
            }),
        )
            .into_response();
    }

    let has_membership =
        match repository::has_active_organization_membership(&app_state.db_pool, user.id).await {
            Ok(has_membership) => has_membership,

            Err(error) => {
                eprintln!("Failed to check onboarding membership: {error}");

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    if !has_membership {
        return StatusCode::FORBIDDEN.into_response();
    }

    (
        StatusCode::OK,
        Json(crate::models::ClientOnboardingStatusResponse {
            status: "active".to_string(),
            onboarding_complete: true,
            user_id: Some(user.id),
        }),
    )
        .into_response()
}

pub async fn create_organization_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Json(payload): Json<crate::models::CreateOrganizationRequest>,
) -> impl IntoResponse {
    // Only ORBI platform administrators may create organizations
    // through the management API.
    //
    // Customers create their initial organization through onboarding.
    if orbi_user.platform_role.as_deref() != Some("SUPER_ADMIN") {
        return (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to create organizations.".to_string(),
            }),
        )
            .into_response();
    }

    match repository::create_organization(&app_state.db_pool, &payload).await {
        Ok(organization_id) => (
            StatusCode::CREATED,
            Json(crate::models::OrganizationMutationResponse {
                organization_id,
                message: "Organization created successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to create organization: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create organization.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn update_organization_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Path(organization_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateOrganizationRequest>,
) -> impl IntoResponse {
    if orbi_user.platform_role.as_deref() != Some("SUPER_ADMIN") {
        return (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to update organizations.".to_string(),
            }),
        )
            .into_response();
    }

    match repository::update_organization(&app_state.db_pool, organization_id, &payload).await {
        Ok(_) => (
            StatusCode::OK,
            Json(crate::models::OrganizationMutationResponse {
                organization_id,
                message: "Organization updated successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to update organization: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to update organization.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn delete_organization_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Path(organization_id): Path<Uuid>,
) -> impl IntoResponse {
    if orbi_user.platform_role.as_deref() != Some("SUPER_ADMIN") {
        return (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to deactivate organizations.".to_string(),
            }),
        )
            .into_response();
    }

    match repository::archive_organization(&app_state.db_pool, organization_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(crate::models::OrganizationMutationResponse {
                organization_id,
                message: "Organization deactivated successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to deactivate organization: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to deactivate organization.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn create_asset_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Json(payload): Json<crate::models::CreateAssetRequest>,
) -> impl IntoResponse {
    match repository::create_asset(&app_state.db_pool, &payload, &orbi_user).await {
        Ok(Some(asset_id)) => (
            StatusCode::CREATED,
            Json(crate::models::AssetMutationResponse {
                asset_id,
                message: "Asset created successfully.".to_string(),
            }),
        )
            .into_response(),

        Ok(None) => (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to create an asset in this organization."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to create asset: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create asset.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn update_asset_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Path(asset_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateAssetRequest>,
) -> impl IntoResponse {
    match repository::update_asset(&app_state.db_pool, asset_id, &payload, &orbi_user).await {
        Ok(true) => (
            StatusCode::OK,
            Json(crate::models::AssetMutationResponse {
                asset_id,
                message: "Asset updated successfully.".to_string(),
            }),
        )
            .into_response(),

        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to update this asset, or it is unavailable."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to update asset: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to update asset.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn delete_asset_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    match repository::archive_asset(&app_state.db_pool, asset_id, &orbi_user).await {
        Ok(true) => (
            StatusCode::OK,
            Json(crate::models::AssetMutationResponse {
                asset_id,
                message: "Asset deactivated successfully.".to_string(),
            }),
        )
            .into_response(),

        Ok(false) => (
            StatusCode::FORBIDDEN,
            Json(crate::models::ApiErrorResponse {
                message: "You are not authorized to deactivate this asset, or it is unavailable."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!("Failed to deactivate asset: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to deactivate asset.".to_string(),
                }),
            )
                .into_response()
        }
    }
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
                || message.contains("has an activation entitlement")
                || message.contains("must be READY_FOR_DEPLOYMENT")
                || message.contains("inactive asset")
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

//--------Device Activation Entitlement Management

pub async fn create_device_activation_entitlement_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<crate::models::CreateDeviceActivationEntitlementRequest>,
) -> impl IntoResponse {
    match device_activation_repository::create_device_activation_entitlement(
        &app_state.db_pool,
        &payload,
    )
    .await
    {
        Ok(entitlement) => (StatusCode::CREATED, Json(entitlement)).into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("Inventory device not found")
                || message.contains("Organization not found")
            {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Inventory device is not ready for deployment")
                || message.contains("Organization is inactive")
            {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("device_activation_entitlements_inventory_device_id_key") {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message:
                            "An activation entitlement already exists for this inventory device."
                                .to_string(),
                    }),
                )
                    .into_response();
            }

            eprintln!("Failed to create device activation entitlement: {}", error);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to create device activation entitlement.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn activate_device_handler(
    State(app_state): State<AppState>,
    Extension(orbi_user): Extension<OrbiUser>,
    Json(payload): Json<crate::models::ActivateDeviceRequest>,
) -> impl IntoResponse {
    let result =
        device_activation_repository::activate_device(&app_state.db_pool, orbi_user.id, &payload)
            .await;

    match result {
        Ok(device_id) => (StatusCode::CREATED, Json(device_id)).into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("not authorized to activate devices") {
                return (
                    StatusCode::FORBIDDEN,
                    Json(crate::models::ApiErrorResponse {
                        message: "You are not authorized to activate this device.".to_string(),
                    }),
                )
                    .into_response();
            }

            if message.contains("Inventory device not found")
                || message.contains("Device activation entitlement not found")
            {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse {
                        message: "Device activation record not found.".to_string(),
                    }),
                )
                    .into_response();
            }

            if message.contains("not pending")
                || message.contains("READY_FOR_DEPLOYMENT")
                || message.contains("no longer eligible for activation")
                || message.contains("Unknown inventory status")
                || message.contains("Asset does not belong")
            {
                return (
                    StatusCode::CONFLICT,
                    Json(crate::models::ApiErrorResponse {
                        message: "Device activation requirements are not satisfied.".to_string(),
                    }),
                )
                    .into_response();
            }

            eprintln!("Device activation failed: {error}");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Unable to activate device.".to_string(),
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

pub async fn get_fuel_calibration_profile_handler(
    State(app_state): State<AppState>,
    Path(sensor_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::get_profile(&app_state.db_pool, sensor_id)
        .await
    {
        Ok(Some(profile)) => (StatusCode::OK, Json(profile)).into_response(),

        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(crate::models::ApiErrorResponse {
                message: "No current fuel calibration profile was found for this sensor."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            eprintln!(
                "Failed to load guided fuel calibration profile for sensor {}: {}",
                sensor_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to load fuel calibration profile.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn abandon_fuel_calibration_session_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::abandon_session(
        &app_state.db_pool,
        session_id,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration session abandoned successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("session was not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message
                .contains("Only an active or paused fuel calibration session can be abandoned")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to abandon fuel calibration session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to abandon fuel calibration session.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn supersede_fuel_calibration_profile_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::supersede_profile(
        &app_state.db_pool,
        profile_id,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationProfileMutationResponse {
                profile_id,
                message: "Fuel calibration profile superseded successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("profile was not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("already superseded")
                || message.contains("cannot be superseded while it has an unfinished session")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to supersede fuel calibration profile {}: {}",
                profile_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to supersede fuel calibration profile.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn publish_fuel_calibration_profile_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::publish_profile(
        &app_state.db_pool,
        profile_id,
    )
    .await
    {
        Ok(calibration_id) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationProfileMutationResponse {
                profile_id,
                message: format!(
                    "Fuel calibration profile published successfully as runtime calibration {}.",
                    calibration_id
                ),
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

            if message.contains("superseded")
                || message.contains("At least two resolved calibration points")
                || message.contains("Calibration levels must")
                || message.contains("final calibration point must")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to publish fuel calibration profile {}: {}",
                profile_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to publish fuel calibration profile.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn activate_fuel_calibration_profile_for_production_handler(
    State(app_state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::activate_profile_for_production(
        &app_state.db_pool,
        profile_id,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationProfileMutationResponse {
                profile_id,
                message: "Fuel calibration profile activated for production successfully."
                    .to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            /*
             * A missing profile is a resource lookup failure.
             */
            if message.contains("not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            /*
             * These are valid requests against a profile whose current
             * lifecycle state does not permit production activation.
             *
             * Keep these as client-visible BAD_REQUEST responses so
             * Platform Management can explain exactly why activation
             * was refused.
             */
            if message.contains("superseded")
                || message.contains("low confidence")
                || message.contains("published")
                || message.contains("same sensor")
                || message.contains("fuel calibration")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to activate fuel calibration profile {} for production: {}",
                profile_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to activate fuel calibration profile for production."
                        .to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn get_latest_fuel_sensor_observation_handler(
    State(app_state): State<AppState>,
    Path(sensor_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::get_latest_sensor_observation(
        &app_state.db_pool,
        sensor_id,
    )
    .await
    {
        Ok(Some(observation)) => (StatusCode::OK, Json(observation)).into_response(),

        /*
         * The sensor may legitimately have no physical observation yet.
         *
         * This can happen immediately after installation, before the device
         * has transmitted its first KUM telemetry packet.
         */
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(crate::models::ApiErrorResponse {
                message: "No physical fuel-sensor observation is available yet.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            eprintln!(
                "Failed to retrieve latest physical fuel observation for sensor {}: {}",
                sensor_id, error
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to retrieve latest fuel-sensor observation.".to_string(),
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

pub async fn capture_fuel_calibration_point_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<crate::models::CaptureFuelCalibrationPointRequest>,
) -> impl IntoResponse {
    /*
     * The HTTP layer does not define stability mathematics.
     *
     * It selects ORBI's production KUM stability policy and delegates the
     * complete automatic-capture workflow to the platform service.
     */
    let config = crate::domain::calibration::FuelCalibrationStabilityConfig::production();

    match crate::services::platform::fuel_calibration::capture_stable_point(
        &app_state.db_pool,
        session_id,
        payload.cumulative_change_litres,
        payload.observation_started_at,
        config,
    )
    .await
    {
        Ok(result) => {
            let stability = result.stability;

            /*
             * Translate the domain state into a stable API representation
             * suitable for the frontend calibration animation.
             */
            let state = match stability.state {
                crate::domain::calibration::FuelCalibrationStabilityState::WaitingForTelemetry => {
                    "waiting_for_telemetry"
                }

                crate::domain::calibration::FuelCalibrationStabilityState::Observing => "observing",

                crate::domain::calibration::FuelCalibrationStabilityState::Settling => "settling",

                crate::domain::calibration::FuelCalibrationStabilityState::Stable => "stable",
            };

            let captured = result.point_id.is_some();

            let message = match stability.state {
                crate::domain::calibration::FuelCalibrationStabilityState::WaitingForTelemetry => {
                    "Waiting for fuel sensor telemetry."
                }

                crate::domain::calibration::FuelCalibrationStabilityState::Observing => {
                    "Observing fuel sensor measurements."
                }

                crate::domain::calibration::FuelCalibrationStabilityState::Settling => {
                    "Fuel level is settling."
                }

                crate::domain::calibration::FuelCalibrationStabilityState::Stable => {
                    "Fuel level is stable. Calibration point captured automatically."
                }
            };

            /*
             * Every successful evaluation returns 200 OK.
             *
             * The endpoint is intentionally pollable and idempotent:
             *
             * observing -> 200
             * settling  -> 200
             * stable    -> 200 + point_id
             *
             * Repeated stable requests for the same cumulative position
             * return the existing calibration point.
             */
            (
                StatusCode::OK,
                Json(crate::models::FuelCalibrationAutomaticCaptureResponse {
                    state: state.to_string(),

                    sample_count: stability.sample_count,

                    observation_duration_seconds: stability.observation_duration_ms as f64 / 1000.0,

                    realtime_range_cm: stability.realtime_range_cm,

                    realtime_slope_cm_per_second: stability.realtime_slope_cm_per_second,

                    capture_distance_cm: stability.capture_distance_cm,

                    captured,
                    point_id: result.point_id,

                    message: message.to_string(),
                }),
            )
                .into_response()
        }

        Err(error) => {
            let message = error.to_string();

            if message.contains("session was not found") || message.contains("profile not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Cumulative fuel change")
                || message.contains("can only be captured while the session is active")
                || message.contains("outside the declared tank capacity")
                || message
                    .contains("Stable fuel calibration result did not contain a capture distance")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed automatic fuel calibration capture for session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to evaluate automatic fuel calibration capture.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn pause_fuel_calibration_session_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::pause_session(&app_state.db_pool, session_id)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration session paused successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            eprintln!(
                "Failed to pause fuel calibration session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to pause fuel calibration session.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn resume_fuel_calibration_session_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::resume_session(
        &app_state.db_pool,
        session_id,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration session resumed successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            eprintln!(
                "Failed to resume fuel calibration session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to resume fuel calibration session.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn apply_fuel_calibration_anchor_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(payload): Json<crate::models::ApplyFuelCalibrationAnchorRequest>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::apply_anchor(
        &app_state.db_pool,
        session_id,
        payload.cumulative_change_litres,
        payload.absolute_litres,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration anchor applied successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("session was not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Only an active or paused")
                || message.contains("cannot be applied before")
                || message.contains("must correspond to a captured calibration point")
                || message.contains("Tank capacity")
                || message.contains("Calibration anchor")
                || message.contains("resolves outside the declared tank capacity")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to apply fuel calibration anchor for session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to apply fuel calibration anchor.".to_string(),
                }),
            )
                .into_response()
        }
    }
}

pub async fn complete_fuel_calibration_session_handler(
    State(app_state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> impl IntoResponse {
    match crate::services::platform::fuel_calibration::complete_session(
        &app_state.db_pool,
        session_id,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(crate::models::FuelCalibrationSessionMutationResponse {
                session_id,
                message: "Fuel calibration session completed successfully.".to_string(),
            }),
        )
            .into_response(),

        Err(error) => {
            let message = error.to_string();

            if message.contains("session was not found") {
                return (
                    StatusCode::NOT_FOUND,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            if message.contains("Only an active or paused")
                || message.contains("cannot be completed")
                || message.contains("at least two verified points")
                || message.contains("remain unresolved")
                || message.contains("outside the declared tank capacity")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(crate::models::ApiErrorResponse { message }),
                )
                    .into_response();
            }

            eprintln!(
                "Failed to complete fuel calibration session {}: {}",
                session_id, message
            );

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(crate::models::ApiErrorResponse {
                    message: "Failed to complete fuel calibration session.".to_string(),
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
