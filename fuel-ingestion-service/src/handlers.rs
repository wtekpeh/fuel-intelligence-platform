use crate::{routes::AppState, services::telemetry::fuel_service::persist_fuel_reading};
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::input_adapters::legacy_fuel_reading::map_legacy_readings;
use crate::services::fuel_detection::{detect_fuel_event, detect_possible_leak};
use crate::services::sensor_health::detect_frozen_fuel_sensor;
use crate::services::telemetry::gps_service::persist_gps_reading;
use crate::{
    models::{
        AlertResponse, AnalyticsDeviceHealthTrendQuery, AnalyticsGeofenceActivityQuery,
        AnalyticsGeofenceUtilizationQuery, ApiResponse, CheckPositionRequest,
        CheckPositionResponse, CreateGeofenceRequest, DeviceHealthTrendResponse,
        DeviceStateEventResponse, GeofenceActivityTrendResponse, GeofenceUtilizationResponse,
        HeartbeatRequest, HeartbeatResponse, ReadingBatch,
    },
    repository::{
        acknowledge_alert, check_position_against_geofences, create_geofence,
        detect_and_store_geofence_transitions_from_previous_position,
        find_registered_telemetry_context, get_alert_trends, get_device_health_trends,
        get_geofence_activity_trends, get_geofence_utilization, get_organization_fleet_overview,
        get_organization_id_for_device, get_organization_overview, get_recent_alerts,
        get_recent_device_health_events, get_recent_device_state_events, get_recent_fuel_events,
        get_recent_sensor_health_events, get_recent_telemetry_stream, get_telemetry_history,
        list_geofences, list_recent_geofence_transition_events, mark_device_heartbeat_seen,
        mark_device_payload_seen, refresh_device_statuses, resolve_alert,
    },
};

pub async fn ingest_reading_batch(
    State(app_state): State<AppState>,
    Json(payload): Json<ReadingBatch>,
) -> impl IntoResponse {
    let received_count = payload.readings.len();

    // Build the canonical ORBI telemetry model.
    //
    // For now this runs alongside the existing ingestion pipeline.
    // The operational services still consume FuelReading until
    // they are migrated individually.
    let telemetry_readings = map_legacy_readings(&payload.readings);

    // Prevent an unused-variable warning while the migration
    // is still in progress.
    let _ = &telemetry_readings;

    println!("Received batch from device: {}", payload.device_id);
    println!("Synced at: {}", payload.synced_at);
    println!("Readings received: {}", received_count);

    let db_pool = &app_state.db_pool;
    let result = async {
        let context = find_registered_telemetry_context(db_pool, &payload.device_id).await?;

        let Some(context) = context else {
            return Err(anyhow::anyhow!(
                "Unknown device '{}'. Device must be provisioned before telemetry can be ingested.",
                payload.device_id
            ));
        };

        let device_id = context.device_id;

        mark_device_payload_seen(db_pool, device_id).await?;

        let mut previous_reading: Option<&crate::models::FuelReading> = None;

        let organization_id = get_organization_id_for_device(db_pool, device_id).await?;

        for reading in &payload.readings {
            // Store GPS observations when this device has a GPS sensor.
            if let Some(gps_sensor_id) = context.gps_sensor_id {
                persist_gps_reading(db_pool, device_id, gps_sensor_id, reading).await?;
            }

            // Preserve geofence intelligence for GPS-capable devices.
            if context.gps_sensor_id.is_some() {
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
            }

            // Run Fuel Intelligence only when the device has a fuel sensor.
            if let Some(fuel_sensor_id) = context.fuel_sensor_id {
                persist_fuel_reading(
                    db_pool,
                    device_id,
                    fuel_sensor_id,
                    reading,
                    previous_reading,
                )
                .await?;

                detect_fuel_event(
                    db_pool,
                    &app_state.alert_hub,
                    &app_state.config,
                    device_id,
                    fuel_sensor_id,
                )
                .await?;

                detect_possible_leak(
                    db_pool,
                    &app_state.alert_hub,
                    &app_state.config,
                    device_id,
                    fuel_sensor_id,
                )
                .await?;

                detect_frozen_fuel_sensor(db_pool, device_id, fuel_sensor_id).await?;
            }

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

pub async fn get_alert_trends_handler(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::AnalyticsAlertTrendQuery>,
) -> impl IntoResponse {
    let requested_days = query.days.unwrap_or(30);

    match get_alert_trends(&app_state.db_pool, query.device_id, requested_days).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),

        Err(err) => {
            eprintln!("Failed to fetch alert trends: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch alert trends"
                })),
            )
                .into_response()
        }
    }
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

pub async fn list_geofence_transition_events_handler(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryQueryParams>,
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match list_recent_geofence_transition_events(db_pool, query.device_id).await {
        Ok(events) => (StatusCode::OK, Json(events)).into_response(),

        Err(err) => {
            eprintln!("Failed to fetch geofence transition events: {}", err);

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "Failed to fetch geofence transition events"
                })),
            )
                .into_response()
        }
    }
}

pub async fn list_telemetry_history(
    State(app_state): State<AppState>,
    Query(query): Query<crate::models::TelemetryHistoryQueryParams>,
) -> Result<Json<Vec<crate::models::TelemetryStreamResponse>>, StatusCode> {
    if query.start_time > query.end_time {
        return Err(StatusCode::BAD_REQUEST);
    }

    let readings = get_telemetry_history(
        &app_state.db_pool,
        query.device_id,
        query.start_time,
        query.end_time,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(readings))
}

pub async fn get_geofence_activity_trends_handler(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsGeofenceActivityQuery>,
) -> Result<Json<GeofenceActivityTrendResponse>, StatusCode> {
    let days = query.days.unwrap_or(30);

    let response = get_geofence_activity_trends(&state.db_pool, query.device_id, days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(response))
}

pub async fn get_device_health_trends_handler(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsDeviceHealthTrendQuery>,
) -> Result<Json<DeviceHealthTrendResponse>, StatusCode> {
    let days = query.days.unwrap_or(30);

    let response = get_device_health_trends(&state.db_pool, days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(response))
}

pub async fn get_geofence_utilization_handler(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsGeofenceUtilizationQuery>,
) -> Result<Json<GeofenceUtilizationResponse>, StatusCode> {
    let days = query.days.unwrap_or(30);

    let response = get_geofence_utilization(&state.db_pool, days)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(response))
}
