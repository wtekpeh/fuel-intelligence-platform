use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::CreateOrbiDeviceInventoryRequest;

pub async fn create_orbi_inventory_device(
    db_pool: &PgPool,
    request: &CreateOrbiDeviceInventoryRequest,
) -> Result<Uuid> {
    let inventory_device_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO orbi_device_inventory (
            id,
            device_code,
            serial_number,
            imei,
            device_model_id,
            hardware_profile_id,
            firmware_version,
            production_batch,
            notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        inventory_device_id,
        request.device_code,
        request.serial_number,
        request.imei,
        request.device_model_id,
        request.hardware_profile_id,
        request.firmware_version,
        request.production_batch,
        request.notes,
    )
    .execute(db_pool)
    .await?;

    Ok(inventory_device_id)
}

pub async fn list_orbi_inventory_devices(
    db_pool: &PgPool,
) -> Result<Vec<crate::models::OrbiDeviceInventory>> {
    let devices = sqlx::query_as!(
        crate::models::OrbiDeviceInventory,
        r#"
        SELECT
            id,
            device_code,
            serial_number,
            imei,
            device_model_id,
            hardware_profile_id,
            firmware_version,
            production_batch,
            inventory_status,
            quality_test_status,
            notes,
            created_at,
            updated_at
        FROM orbi_device_inventory
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db_pool)
    .await?;

    Ok(devices)
}

pub async fn verify_orbi_inventory_device_by_code(
    db_pool: &PgPool,
    device_code: &str,
) -> Result<Option<crate::models::OrbiDeviceInventory>> {
    let device = sqlx::query_as!(
        crate::models::OrbiDeviceInventory,
        r#"
        SELECT
            id,
            device_code,
            serial_number,
            imei,
            device_model_id,
            hardware_profile_id,
            firmware_version,
            production_batch,
            inventory_status,
            quality_test_status,
            notes,
            created_at,
            updated_at
        FROM orbi_device_inventory
        WHERE device_code = $1
        LIMIT 1
        "#,
        device_code,
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(device)
}

pub async fn update_orbi_inventory_status(
    db_pool: &PgPool,
    inventory_device_id: Uuid,
    inventory_status: &str,
    quality_test_status: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE orbi_device_inventory
        SET
            inventory_status = $2,
            quality_test_status = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
        inventory_device_id,
        inventory_status,
        quality_test_status,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_orbi_inventory_device(
    db_pool: &PgPool,
    inventory_device_id: Uuid,
) -> Result<Option<crate::models::OrbiDeviceInventory>> {
    let device = sqlx::query_as!(
        crate::models::OrbiDeviceInventory,
        r#"
        SELECT
            id,
            device_code,
            serial_number,
            imei,
            device_model_id,
            hardware_profile_id,
            firmware_version,
            production_batch,
            inventory_status,
            quality_test_status,
            notes,
            created_at,
            updated_at
        FROM orbi_device_inventory
        WHERE id = $1
        "#,
        inventory_device_id,
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(device)
}
