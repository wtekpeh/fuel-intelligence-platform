use axum::{Router, routing::post};
use sqlx::PgPool;

use crate::handlers::ingest_reading_batch;

pub fn app_routes(db_pool: PgPool) -> Router {
    Router::new()
        .route("/api/fuel-readings/batch", post(ingest_reading_batch))
        .with_state(db_pool)
}
