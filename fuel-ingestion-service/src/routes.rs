use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::handlers::{ingest_reading_batch, list_recent_fuel_events};

pub fn app_routes(db_pool: PgPool) -> Router {
    Router::new()
        .route("/api/fuel-readings/batch", post(ingest_reading_batch))
        .route("/api/fuel-events", get(list_recent_fuel_events))
        .with_state(db_pool)
}
