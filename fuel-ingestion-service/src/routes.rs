use crate::config::AppConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::domain::telemetry::telemetry_pipeline::TelemetryPipeline;
use crate::handlers::{
    acknowledge_alert_handler, check_position_against_geofences_handler, create_geofence_handler,
    get_alert_trends_handler, get_device_health_trends_handler,
    get_geofence_activity_trends_handler, get_geofence_utilization_handler, ingest_reading_batch,
    list_alerts, list_device_state_events, list_geofence_transition_events_handler,
    list_geofences_handler, list_organization_fleet_overview, list_organization_overview,
    list_recent_device_health_events, list_recent_fuel_events, list_recent_sensor_health_events,
    list_recent_telemetry_stream, list_telemetry_history, receive_heartbeat, refresh_device_health,
    resolve_alert_handler,
};
use crate::platform_routes::platform_routes;
use crate::services::alert_hub::AlertHub;
use crate::ws::alerts_ws_handler;
use axum::http::{Method, header};
use axum::{
    Router,
    routing::{get, patch, post},
};
use sqlx::PgPool;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub config: AppConfig,
    pub alert_hub: AlertHub,
    pub telemetry_pipeline: Arc<Mutex<TelemetryPipeline>>,
}

pub fn app_routes(db_pool: PgPool, config: AppConfig, alert_hub: AlertHub) -> Router {
    let app_state = AppState {
        db_pool,
        config,
        alert_hub,
        telemetry_pipeline: Arc::new(Mutex::new(TelemetryPipeline::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _request_parts| {
            let origin_string = match origin.to_str() {
                Ok(value) => value,
                Err(_) => return false,
            };

            origin_string == "http://127.0.0.1:5173"
                || origin_string == "http://localhost:5173"
                || origin_string == "https://williamtekpeh.com"
                || origin_string == "http://williamtekpeh.com"
                || origin_string.ends_with(".williamtekpeh.com")
        }))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .merge(platform_routes())
        .route("/api/fuel-readings/batch", post(ingest_reading_batch))
        .route("/api/fuel-events", get(list_recent_fuel_events))
        .route(
            "/api/fuel-readings/recent",
            get(list_recent_telemetry_stream),
        )
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
        .route("/api/analytics/alert-trends", get(get_alert_trends_handler))
        .route(
            "/api/analytics/device-health-trends",
            get(get_device_health_trends_handler),
        )
        .route(
            "/api/analytics/geofence-utilization",
            get(get_geofence_utilization_handler),
        )
        .route(
            "/api/analytics/geofence-activity",
            get(get_geofence_activity_trends_handler),
        )
        .route(
            "/api/alerts/:alert_id/acknowledge",
            patch(acknowledge_alert_handler),
        )
        .route(
            "/api/alerts/:alert_id/resolve",
            patch(resolve_alert_handler),
        )
        .route(
            "/api/organizations/overview",
            get(list_organization_overview),
        )
        .route(
            "/api/organizations/:organization_id/fleet-overview",
            get(list_organization_fleet_overview),
        )
        .route("/api/geofences", post(create_geofence_handler))
        .route(
            "/api/geofences/:organization_id",
            get(list_geofences_handler),
        )
        .route(
            "/api/geofences/check-position",
            post(check_position_against_geofences_handler),
        )
        .route(
            "/api/geofence-transition-events",
            get(list_geofence_transition_events_handler),
        )
        .route("/api/fuel-readings/history", get(list_telemetry_history))
        .route("/ws/alerts", get(alerts_ws_handler))
        .with_state(app_state)
        .layer(cors)
}
