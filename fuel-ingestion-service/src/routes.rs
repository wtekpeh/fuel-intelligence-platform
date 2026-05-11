use crate::config::AppConfig;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::handlers::{
    ingest_reading_batch, list_recent_device_health_events, list_recent_fuel_events,
    receive_heartbeat, refresh_device_health,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: AppConfig,
}

pub fn app_routes(db_pool: PgPool, config: AppConfig) -> Router {
    let app_state = AppState { db_pool, config };

    Router::new()
        .route("/api/fuel-readings/batch", post(ingest_reading_batch))
        .route("/api/fuel-events", get(list_recent_fuel_events))
        .route("/api/heartbeat", post(receive_heartbeat))
        .route("/api/devices/refresh-health", post(refresh_device_health))
        .route(
            "/api/device-health-events",
            get(list_recent_device_health_events),
        )
        .with_state(app_state)
}
