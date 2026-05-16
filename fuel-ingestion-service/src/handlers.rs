use crate::routes::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::services::fuel_detection::{detect_fuel_event, detect_possible_leak};
use crate::services::sensor_health::detect_frozen_fuel_sensor;
use crate::{
    models::{
        ApiResponse, DeviceStateEventResponse, HeartbeatRequest, HeartbeatResponse, ReadingBatch,
    },
    repository::{
        get_or_create_demo_sensor, get_recent_device_health_events, get_recent_device_state_events,
        get_recent_fuel_events, get_recent_sensor_health_events, mark_device_heartbeat_seen,
        mark_device_payload_seen, refresh_device_statuses, save_fuel_reading_as_sensor_reading,
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

        for reading in &payload.readings {
            save_fuel_reading_as_sensor_reading(
                db_pool,
                device_id,
                sensor_id,
                reading,
                previous_reading,
            )
            .await?;

            detect_fuel_event(db_pool, device_id, sensor_id).await?;

            detect_possible_leak(db_pool, device_id, sensor_id).await?;

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

pub async fn list_recent_fuel_events(State(app_state): State<AppState>) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_fuel_events(db_pool, 50).await {
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
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_device_health_events(db_pool, 50).await {
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
) -> impl IntoResponse {
    let db_pool = &app_state.db_pool;

    match get_recent_sensor_health_events(db_pool, 50).await {
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
) -> Result<Json<Vec<DeviceStateEventResponse>>, StatusCode> {
    let events = get_recent_device_state_events(&app_state.db_pool, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}
