use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;

use crate::{
    models::{ApiResponse, ReadingBatch},
    repository::{get_or_create_demo_sensor, save_fuel_reading_as_sensor_reading},
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

        for reading in &payload.readings {
            save_fuel_reading_as_sensor_reading(&db_pool, device_id, sensor_id, reading).await?;
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
