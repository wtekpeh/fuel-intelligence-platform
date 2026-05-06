use axum::{extract::Json, response::IntoResponse};

use crate::models::{ApiResponse, ReadingBatch};

pub async fn ingest_reading_batch(
    Json(payload): Json<ReadingBatch>,
) -> impl IntoResponse {
    let received_count = payload.readings.len();

    println!("Received batch from device: {}", payload.device_id);
    println!("Synced at: {}", payload.synced_at);
    println!("Readings received: {}", received_count);

    for reading in &payload.readings {
        println!(
            "{} | {}L | {}% | mode: {}",
            reading.timestamp,
            reading.fuel_level_litres,
            reading.fuel_level_percentage,
            reading.simulation_mode
        );
    }

    Json(ApiResponse {
        success: true,
        message: "Batch received successfully".to_string(),
        received_count,
    })
}