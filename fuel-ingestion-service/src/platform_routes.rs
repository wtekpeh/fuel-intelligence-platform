use crate::platform_handlers::{
    list_device_sensors_handler, list_devices_handler, list_hardware_profile_sensors_handler,
    list_hardware_profiles_handler, register_device_handler,
};
use crate::routes::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub fn platform_routes() -> Router<AppState> {
    Router::new()
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
