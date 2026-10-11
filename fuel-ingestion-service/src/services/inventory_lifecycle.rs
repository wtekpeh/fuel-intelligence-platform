use anyhow::{Result, anyhow};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::inventory_status::InventoryStatus;
use crate::orbi_inventory_repository;

fn validate_inventory_transition(
    current_status: InventoryStatus,
    next_status: InventoryStatus,
    quality_test_status: &str,
) -> Result<()> {
    if !current_status.can_transition_to(next_status) {
        return Err(anyhow!(
            "Invalid inventory lifecycle transition: {} → {}",
            current_status.as_str(),
            next_status.as_str()
        ));
    }

    if next_status == InventoryStatus::ReadyForDeployment && quality_test_status != "PASSED" {
        return Err(anyhow!(
            "Device cannot become READY_FOR_DEPLOYMENT until quality testing has PASSED."
        ));
    }

    Ok(())
}

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

    validate_inventory_transition(current_status, next_status, quality_test_status)?;

    orbi_inventory_repository::update_orbi_inventory_status(
        db_pool,
        inventory_device_id,
        current_status.as_str(),
        next_status.as_str(),
        quality_test_status,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_deployment_when_quality_test_passed() {
        let result = validate_inventory_transition(
            InventoryStatus::Tested,
            InventoryStatus::ReadyForDeployment,
            "PASSED",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_deployment_when_quality_test_pending() {
        let result = validate_inventory_transition(
            InventoryStatus::Tested,
            InventoryStatus::ReadyForDeployment,
            "PENDING",
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("quality testing has PASSED")
        );
    }

    #[test]
    fn rejects_deployment_when_quality_test_failed() {
        let result = validate_inventory_transition(
            InventoryStatus::Tested,
            InventoryStatus::ReadyForDeployment,
            "FAILED",
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_inventory_transition() {
        let result = validate_inventory_transition(
            InventoryStatus::Assembled,
            InventoryStatus::ReadyForDeployment,
            "PASSED",
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid inventory lifecycle transition")
        );
    }
}
