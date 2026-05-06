use axum::{routing::post, Router};

use crate::handlers::ingest_reading_batch;

pub fn app_routes() -> Router {
    Router::new().route("/api/fuel-readings/batch", post(ingest_reading_batch))
}
