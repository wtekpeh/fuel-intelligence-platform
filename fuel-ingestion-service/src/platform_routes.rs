use crate::platform_handlers::{
    create_asset_handler, create_organization_handler, delete_asset_handler,
    delete_organization_handler, list_device_sensors_handler, list_devices_handler,
    list_hardware_profile_sensors_handler, list_hardware_profiles_handler, register_device_handler,
    update_asset_handler, update_organization_handler,
};
use crate::routes::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn platform_routes() -> Router<AppState> {
    Router::new()
        .route("/api/organizations", post(create_organization_handler))
        .route(
            "/api/organizations/:organization_id",
            axum::routing::patch(update_organization_handler),
        )
        .route("/api/assets", post(create_asset_handler))
        .route(
            "/api/assets/:asset_id",
            axum::routing::patch(update_asset_handler),
        )
        .route(
            "/api/assets/:asset_id",
            axum::routing::delete(delete_asset_handler),
        )
        .route(
            "/api/organizations/:organization_id",
            axum::routing::delete(delete_organization_handler),
        )
        .route(
            "/api/hardware-profiles",
            get(list_hardware_profiles_handler),
        )
        .route(
            "/api/hardware-profiles/:hardware_profile_id/sensors",
            get(list_hardware_profile_sensors_handler),
        )
        .route("/api/devices", post(register_device_handler))
        .route("/api/devices", get(list_devices_handler))
        .route(
            "/api/devices/:device_id/sensors",
            get(list_device_sensors_handler),
        )
}
