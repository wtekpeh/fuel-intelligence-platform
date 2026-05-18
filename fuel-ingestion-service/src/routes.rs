use crate::config::AppConfig;
use crate::handlers::{
    acknowledge_alert_handler, ingest_reading_batch, list_alerts, list_device_state_events,
    list_recent_device_health_events, list_recent_fuel_events, list_recent_sensor_health_events,
    receive_heartbeat, refresh_device_health,
};
use crate::services::alert_hub::AlertHub;
use crate::ws::alerts_ws_handler;
use axum::{
    Router,
    routing::{get, patch, post},
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: AppConfig,
    pub alert_hub: AlertHub,
}

pub fn app_routes(db_pool: PgPool, config: AppConfig, alert_hub: AlertHub) -> Router {
    let app_state = AppState {
        db_pool,
        config,
        alert_hub,
    };

    Router::new()
        .route("/api/fuel-readings/batch", post(ingest_reading_batch))
        .route("/api/fuel-events", get(list_recent_fuel_events))
        .route("/api/heartbeat", post(receive_heartbeat))
        .route("/api/devices/refresh-health", post(refresh_device_health))
        .route(
            "/api/device-health-events",
            get(list_recent_device_health_events),
        )
        .route(
            "/api/sensor-health-events",
            get(list_recent_sensor_health_events),
        )
        .route("/api/device-state-events", get(list_device_state_events))
        .route("/api/alerts", get(list_alerts))
        .route(
            "/api/alerts/:alert_id/acknowledge",
            patch(acknowledge_alert_handler),
        )
        .route("/ws/alerts", get(alerts_ws_handler))
        .with_state(app_state)
}
