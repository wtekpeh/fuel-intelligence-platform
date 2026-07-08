use crate::platform_handlers::{
    assign_device_asset_handler, create_asset_handler, create_orbi_inventory_device_handler,
    create_organization_handler, delete_asset_handler, delete_device_handler,
    delete_organization_handler, get_orbi_inventory_device_handler, list_device_catalogue_handler,
    list_device_models_handler, list_device_sensors_handler, list_devices_handler,
    list_hardware_profile_sensors_handler, list_hardware_profiles_handler,
    list_orbi_inventory_devices_handler, provision_inventory_device_handler,
    register_device_handler, update_asset_handler, update_device_handler,
    update_orbi_inventory_status_handler, update_organization_handler,
    verify_orbi_inventory_device_handler,
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
            "/api/devices/:device_id",
            axum::routing::patch(update_device_handler),
        )
        .route(
            "/api/devices/:device_id",
            axum::routing::delete(delete_device_handler),
        )
        .route(
            "/api/devices/:device_id/assign-asset",
            axum::routing::patch(assign_device_asset_handler),
        )
        .route(
            "/api/devices/:device_id/sensors",
            get(list_device_sensors_handler),
        )
        .route("/api/device-models", get(list_device_models_handler))
        .route("/api/device-catalogue", get(list_device_catalogue_handler))
        //Orbi Inventory
        .route(
            "/api/device-inventory",
            post(create_orbi_inventory_device_handler).get(list_orbi_inventory_devices_handler),
        )
        .route(
            "/api/device-inventory/verify/:device_code",
            get(verify_orbi_inventory_device_handler),
        )
        .route(
            "/api/device-inventory/:inventory_device_id/status",
            axum::routing::patch(update_orbi_inventory_status_handler),
        )
        .route(
            "/api/device-inventory/:inventory_device_id",
            get(get_orbi_inventory_device_handler),
        )
        .route(
            "/api/devices/provision-from-inventory",
            post(provision_inventory_device_handler),
        )
}
