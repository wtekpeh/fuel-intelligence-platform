use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::services::fuel_detection::{detect_fuel_event, detect_possible_leak};
use crate::{
    models::{ApiResponse, ReadingBatch},
    repository::{
        get_or_create_demo_sensor, get_recent_fuel_events, mark_device_payload_seen,
        save_fuel_reading_as_sensor_reading,
    },
};

pub async fn ingest_reading_batch(
    State(db_pool): State<PgPool>,
    Json(payload): Json<ReadingBatch>,
) -> impl IntoResponse {
    let received_count = payload.readings.len();

    println!("Received batch from device: {}", payload.device_id);
    println!("Synced at: {}", payload.synced_at);
    println!("Readings received: {}", received_count);

    let result = async {
        let (device_id, sensor_id) =
            get_or_create_demo_sensor(&db_pool, &payload.device_id).await?;

        mark_device_payload_seen(&db_pool, device_id).await?;

        for reading in &payload.readings {
            save_fuel_reading_as_sensor_reading(&db_pool, device_id, sensor_id, reading).await?;

            detect_fuel_event(&db_pool, device_id, sensor_id).await?;

            detect_possible_leak(&db_pool, device_id, sensor_id).await?;
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

pub async fn list_recent_fuel_events(State(db_pool): State<PgPool>) -> impl IntoResponse {
    match get_recent_fuel_events(&db_pool, 50).await {
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
