use anyhow::{Result, anyhow};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ActivateDeviceRequest, CreateDeviceActivationEntitlementRequest, DeviceActivationEntitlement,
};

pub async fn create_device_activation_entitlement(
    db_pool: &PgPool,
    request: &CreateDeviceActivationEntitlementRequest,
) -> Result<DeviceActivationEntitlement> {
    let mut tx = db_pool.begin().await?;

    // Lock the inventory device while validating assignment eligibility.
    let inventory = sqlx::query!(
        r#"
        SELECT inventory_status
        FROM orbi_device_inventory
        WHERE id = $1
        FOR UPDATE
        "#,
        request.inventory_device_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow!("Inventory device not found."))?;

    if inventory.inventory_status != "READY_FOR_DEPLOYMENT" {
        return Err(anyhow!("Inventory device is not ready for deployment."));
    }

    // Confirm that the customer organization exists and is active.
    let organization = sqlx::query!(
        r#"
        SELECT is_active
        FROM organizations
        WHERE id = $1
        "#,
        request.organization_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow!("Organization not found."))?;

    if !organization.is_active {
        return Err(anyhow!("Organization is inactive."));
    }

    // Insert the entitlement.
    // The UNIQUE constraint on inventory_device_id prevents
    // multiple assignments for the same physical device.
    let entitlement = sqlx::query_as!(
        DeviceActivationEntitlement,
        r#"
        INSERT INTO device_activation_entitlements (
            inventory_device_id,
            organization_id
        )
        VALUES ($1, $2)
        RETURNING
            id,
            inventory_device_id,
            organization_id,
            status,
            activated_device_id,
            created_at,
            activated_at
        "#,
        request.inventory_device_id,
        request.organization_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(entitlement)
}

/// Activate an inventory device for its entitled customer organization.
///
/// All validation and provisioning must occur inside one transaction.
pub async fn activate_device(
    db_pool: &PgPool,
    user_id: Uuid,
    request: &ActivateDeviceRequest,
) -> Result<Uuid> {
    let mut tx = db_pool.begin().await?;

    // 1. Lock the inventory record first.
    // This follows the same locking order used by administrative
    // entitlement assignment and legacy provisioning.
    let inventory = sqlx::query!(
        r#"
        SELECT
            device_code,
            device_model_id,
            hardware_profile_id,
            inventory_status
        FROM orbi_device_inventory
        WHERE id = $1
        FOR UPDATE
        "#,
        request.inventory_device_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow!("Inventory device not found."))?;

    // 2. Retrieve and lock the activation entitlement.
    let entitlement = sqlx::query!(
        r#"
        SELECT
            id,
            organization_id,
            status
        FROM device_activation_entitlements
        WHERE inventory_device_id = $1
        FOR UPDATE
        "#,
        request.inventory_device_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| anyhow!("Device activation entitlement not found."))?;

    if entitlement.status != "PENDING" {
        return Err(anyhow!("Device activation entitlement is not pending."));
    }

    // 3. Validate the inventory lifecycle.
    use crate::domain::inventory_status::InventoryStatus;

    let current_status = InventoryStatus::from_str(&inventory.inventory_status)
        .ok_or_else(|| anyhow!("Unknown inventory status."))?;

    if current_status != InventoryStatus::ReadyForDeployment {
        return Err(anyhow!(
            "Inventory device must be READY_FOR_DEPLOYMENT before activation."
        ));
    }

    // 4. Verify and lock organization-specific activation permissions.
    //
    // Locking the membership, organization, and user rows ensures
    // that concurrent permission revocation or deactivation must
    // coordinate with this activation transaction.
    let authorized = sqlx::query_scalar!(
        r#"
    SELECT TRUE AS "authorized!"
    FROM organization_memberships AS membership
    JOIN organizations
        ON organizations.id = membership.organization_id
    JOIN orbi_users
        ON orbi_users.id = membership.user_id
    WHERE membership.user_id = $1
      AND membership.organization_id = $2
      AND membership.is_active = TRUE
      AND organizations.is_active = TRUE
      AND orbi_users.is_active = TRUE
      AND membership.role IN ('ADMIN', 'FLEET_MANAGER')
    FOR SHARE OF membership, organizations, orbi_users
    "#,
        user_id,
        entitlement.organization_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(false);

    if !authorized {
        return Err(anyhow!(
            "User is not authorized to activate devices for this organization."
        ));
    }

    // 5. Verify and lock the selected asset.
    //
    // The asset must belong to the organization assigned
    // to this device through its activation entitlement.
    //
    // FOR SHARE prevents concurrent updates to this asset
    // until the activation transaction completes.
    let asset_is_valid = sqlx::query_scalar!(
        r#"
    SELECT TRUE AS "is_valid!"
    FROM assets
    WHERE id = $1
      AND organization_id = $2
      AND is_active = TRUE
    FOR SHARE
    "#,
        request.asset_id,
        entitlement.organization_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .unwrap_or(false);

    if !asset_is_valid {
        return Err(anyhow!(
            "Asset does not belong to the entitled organization or is inactive."
        ));
    }

    // 6. Register the operational device and its sensors.
    //
    // The device and all hardware-profile sensors are created
    // inside the existing activation transaction.
    //
    // If any subsequent operation fails, PostgreSQL will roll
    // back the device and sensor records as well.
    let device_id = crate::repository::register_device_tx(
        &mut tx,
        request.asset_id,
        Some(inventory.device_model_id),
        inventory.device_code,
        inventory.hardware_profile_id,
    )
    .await?;

    // 7. Mark the inventory device as PROVISIONED.
    //
    // The conditional UPDATE provides an additional safeguard
    // against invalid inventory lifecycle transitions.
    //
    // We deliberately preserve the existing quality_test_status.
    let updated_inventory = sqlx::query!(
        r#"
    UPDATE orbi_device_inventory
    SET
        inventory_status = 'PROVISIONED',
        updated_at = NOW()
    WHERE id = $1
      AND inventory_status = 'READY_FOR_DEPLOYMENT'
    RETURNING id
    "#,
        request.inventory_device_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if updated_inventory.is_none() {
        return Err(anyhow!(
            "Inventory device is no longer eligible for activation."
        ));
    }

    // 8. Activate the entitlement and associate it with
    // the newly registered operational device.
    //
    // The conditional UPDATE ensures that only a PENDING
    // entitlement can transition to ACTIVATED.
    let updated_entitlement = sqlx::query!(
        r#"
    UPDATE device_activation_entitlements
    SET
        status = 'ACTIVATED',
        activated_device_id = $2,
        activated_at = NOW()
    WHERE id = $1
      AND status = 'PENDING'
    RETURNING id
    "#,
        entitlement.id,
        device_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if updated_entitlement.is_none() {
        return Err(anyhow!(
            "Device activation entitlement is no longer pending."
        ));
    }

    // 9. Commit the entire activation transaction.
    //
    // This makes the operational device, sensors,
    // inventory status, and entitlement activation
    // permanent together.
    tx.commit().await?;

    Ok(device_id)
}
