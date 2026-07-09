use anyhow::{Result, anyhow};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::inventory_status::InventoryStatus;
use crate::orbi_inventory_repository;

pub async fn update_inventory_status(
    db_pool: &PgPool,
    inventory_device_id: Uuid,
    requested_status: &str,
    quality_test_status: &str,
) -> Result<()> {
    let device = orbi_inventory_repository::get_orbi_inventory_device(db_pool, inventory_device_id)
        .await?
        .ok_or_else(|| anyhow!("Inventory device not found."))?;

    let current_status = InventoryStatus::from_str(&device.inventory_status)
        .ok_or_else(|| anyhow!("Unknown current inventory status."))?;

    let next_status = InventoryStatus::from_str(requested_status)
        .ok_or_else(|| anyhow!("Unknown requested inventory status."))?;

    if !current_status.can_transition_to(next_status) {
        return Err(anyhow!(
            "Invalid inventory lifecycle transition: {} → {}",
            current_status.as_str(),
            next_status.as_str()
        ));
    }

    orbi_inventory_repository::update_orbi_inventory_status(
        db_pool,
        inventory_device_id,
        next_status.as_str(),
        quality_test_status,
    )
    .await?;

    Ok(())
}
