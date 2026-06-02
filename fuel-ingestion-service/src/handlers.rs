use crate::routes::AppState;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::fuel_detection::{detect_fuel_event, detect_possible_leak};
use crate::services::sensor_health::detect_frozen_fuel_sensor;
use crate::{
    models::{
        AlertResponse, ApiResponse, CheckPositionRequest, CheckPositionResponse,
        CreateGeofenceRequest, DeviceStateEventResponse, HeartbeatRequest, HeartbeatResponse,
        ReadingBatch,
    },
    repository::{
        acknowledge_alert, check_position_against_geofences, create_geofence,
        detect_and_store_geofence_transitions_from_previous_position, get_or_create_demo_sensor,
        get_organization_fleet_overview, get_organization_id_for_device, get_organization_overview,
        get_recent_alerts, get_recent_device_health_events, get_recent_device_state_events,
        get_recent_fuel_events, get_recent_sensor_health_events, get_recent_telemetry_stream,
        list_geofences, mark_device_heartbeat_seen, mark_device_payload_seen,
        refresh_device_statuses, resolve_alert, save_fuel_reading_as_sensor_reading,
    },
};

pub async fn ingest_reading_batch(
    State(app_state): State<AppState>,
    Json(payload): Json<ReadingBatch>,
) -> impl IntoResponse {
    let received_count = payload.readings.len();

    println!("Received batch from device: {}", payload.device_id);
    println!("Synced at: {}", payload.synced_at);
    println!("Readings received: {}", received_count);

    let db_pool = &app_state.db_pool;
    let result = async {
        let (device_id, sensor_id) = get_or_create_demo_sensor(db_pool, &payload.device_id).await?;

        mark_device_payload_seen(db_pool, device_id).await?;

        let mut previous_reading = None;

        let organization_id = get_organization_id_for_device(db_pool, device_id).await?;

        for reading in &payload.readings {
            save_fuel_reading_as_sensor_reading(
                db_pool,
                device_id,
                sensor_id,
                reading,
                previous_reading,
            )
            .await?;

            if let Some(previous_reading) = previous_reading {
                detect_and_store_geofence_transitions_from_previous_position(
                    db_pool,
                    organization_id,
                    device_id,
                    previous_reading.latitude,
                    previous_reading.longitude,
                    reading.latitude,
                    reading.longitude,
                    reading.timestamp,
                )
                .await?;
            }

            detect_fuel_event(
                db_pool,
                &app_state.alert_hub,
                &app_state.config,
                device_id,
                sensor_id,
            )
            .await?;

            detect_possible_leak(
                db_pool,
                &app_state.alert_hub,
                &app_state.config,
                device_id,
                sensor_id,
            )
            .await?;

            detect_frozen_fuel_sensor(db_pool, device_id, sensor_id).await?;

            previous_reading = Some(reading);
        }

        anyhow::Ok(())
    }
    .await;

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Batch stored successfully".to_string(),
                received_count,
            }),
        )
            .into_response(),

        Err(err) => {
            eprintln!("Failed to store batch: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to store batch: {}", err),
                    received_count: 0,
                }),
            )
                .into_response()
        }
    }
}

pub async fn list_recent_fuel_events(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_fuel_events(db_pool, 50, query.device_id).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),
        Err(err) => {
            eprintln!("Failed to fetch fuel events: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch fuel events"
                })),
            )
                .into_response()
        }
    }
}

pub async fn receive_heartbeat(
    State(app_state): State<AppState>,
    Json(payload): Json<HeartbeatRequest>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match mark_device_heartbeat_seen(db_pool, &payload.device_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HeartbeatResponse {
                success: true,
                message: "Heartbeat received".to_string(),
                device_id: payload.device_id,
            }),
        )
            .into_response(),

        Err(err) => {
            eprintln!("Heartbeat failed: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("Heartbeat failed: {}", err)
                })),
            )
                .into_response()
        }
    }
}

pub async fn refresh_device_health(State(app_state): State<AppState>) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match refresh_device_statuses(
        db_pool,
        app_state.config.device_stale_after_seconds,
        app_state.config.device_offline_after_seconds,
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Device statuses refreshed"
            })),
        )
            .into_response(),

        Err(err) => {
            eprintln!("Device health refresh failed: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": format!("Device health refresh failed: {}", err)
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_recent_device_health_events(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_device_health_events(db_pool, 50, query.device_id).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),

        Err(err) => {
            eprintln!("Failed to fetch device health events: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch device health events"
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_recent_sensor_health_events(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_sensor_health_events(db_pool, 50, query.device_id).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),

        Err(err) => {
            eprintln!("Failed to fetch sensor health events: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch sensor health events"
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_device_state_events(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> Result<Json<Vec<DeviceStateEventResponse>>, StatusCode> {
    let events = get_recent_device_state_events(&app_state.db_pool, 100, query.device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}

pub async fn list_alerts(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::AlertQueryParams>,
) -> Result<Json<Vec<AlertResponse>>, StatusCode> {
    let alerts: Vec<AlertResponse> = get_recent_alerts(&app_state.db_pool, query.device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(alerts))
}

pub async fn acknowledge_alert_handler(
    State(app_state): State<AppState>,
    Path(alert_id): Path<uuid::Uuid>,
) -> Result<Json<crate::models::AlertAcknowledgementResponse>, StatusCode> {
    let acknowledged_alert = acknowledge_alert(&app_state.db_pool, alert_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match acknowledged_alert {
        Some(alert) => {
            app_state.alert_hub.broadcast_acknowledgement(alert.clone());

            Ok(Json(alert))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn resolve_alert_handler(
    State(app_state): State<AppState>,
    Path(alert_id): Path<uuid::Uuid>,
) -> Result<Json<crate::models::AlertAcknowledgementResponse>, StatusCode> {
    let resolved_alert = resolve_alert(&app_state.db_pool, alert_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match resolved_alert {
        Some(alert) => {
            app_state.alert_hub.broadcast_acknowledgement(alert.clone());

            Ok(Json(alert))
        }

        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_recent_telemetry_stream(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> Result<Json<Vec<crate::models::TelemetryStreamResponse>>, StatusCode> {
    let readings = get_recent_telemetry_stream(&app_state.db_pool, query.device_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(readings))
}

pub async fn list_organization_overview(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<crate::models::OrganizationOverviewResponse>>, StatusCode> {
    let overview = get_organization_overview(&app_state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(overview))
}

pub async fn list_organization_fleet_overview(
    State(app_state): State<AppState>,
    Path(organization_id): Path<uuid::Uuid>,
) -> Result<Json<Vec<crate::models::OrganizationFleetOverviewResponse>>, StatusCode> {
    let overview = get_organization_fleet_overview(&app_state.db_pool, organization_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(overview))
}

pub async fn create_geofence_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateGeofenceRequest>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match create_geofence(db_pool, payload).await {
        Ok(geofence) => (StatusCode::CREATED, Json(geofence)).into_response(),

        Err(err) => {
            eprintln!("Failed to create geofence: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to create geofence"
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_geofences_handler(
    State(app_state): State<AppState>,
    Path(organization_id): Path<uuid::Uuid>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match list_geofences(db_pool, organization_id).await {
        Ok(geofences) => (StatusCode::OK, Json(geofences)).into_response(),

        Err(err) => {
            eprintln!("Failed to fetch geofences: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch geofences"
                })),
            )
                .into_response()
        }
    }
}

pub async fn check_position_against_geofences_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CheckPositionRequest>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match check_position_against_geofences(
        db_pool,
        payload.organization_id,
        payload.device_id,
        payload.latitude,
        payload.longitude,
    )
    .await
    {
        Ok(matched_geofences) => {
            let response = CheckPositionResponse {
                inside_geofence: !matched_geofences.is_empty(),
                matched_geofences,
            };

            (StatusCode::OK, Json(response)).into_response()
        }

        Err(err) => {
            eprintln!("Failed to check position against geofences: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to check position against geofences"
                })),
            )
                .into_response()
        }
    }
}
