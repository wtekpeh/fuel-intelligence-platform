use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    DeviceCatalogueHardwareProfileResponse, DeviceCatalogueModelResponse,
    DeviceCatalogueSensorResponse,
};

struct CatalogueRow {
    device_model_id: Uuid,
    model_code: String,
    model_name: String,
    manufacturer: Option<String>,
    model_description: Option<String>,
    model_is_active: bool,

    hardware_profile_id: Uuid,
    profile_code: String,
    profile_name: String,
    profile_description: Option<String>,
    is_default: bool,

    sensor_id: Uuid,
    sensor_type: String,
    unit: String,
}

pub async fn list_device_catalogue(db_pool: &PgPool) -> Result<Vec<DeviceCatalogueModelResponse>> {
    let rows = sqlx::query_as!(
        CatalogueRow,
        r#"
        SELECT
            dm.id AS device_model_id,
            dm.model_code,
            dm.model_name,
            dm.manufacturer,
            dm.description AS model_description,
            dm.is_active AS model_is_active,

            hp.id AS hardware_profile_id,
            hp.profile_code,
            hp.name AS profile_name,
            hp.description AS profile_description,
            dmhp.is_default,

            hps.id AS sensor_id,
            hps.sensor_type,
            hps.unit

        FROM device_models dm

        INNER JOIN device_model_hardware_profiles dmhp
            ON dmhp.device_model_id = dm.id

        INNER JOIN hardware_profiles hp
            ON hp.id = dmhp.hardware_profile_id

        LEFT JOIN hardware_profile_sensors hps
            ON hps.hardware_profile_id = hp.id

        WHERE dm.is_active = TRUE

        ORDER BY
            dm.model_name ASC,
            dmhp.is_default DESC,
            hp.name ASC,
            hps.sensor_type ASC
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let mut catalogue: Vec<DeviceCatalogueModelResponse> = Vec::new();

    for row in rows {
        let model_index = catalogue
            .iter()
            .position(|model| model.id == row.device_model_id);

        let model_index = match model_index {
            Some(index) => index,
            None => {
                catalogue.push(DeviceCatalogueModelResponse {
                    id: row.device_model_id,
                    model_code: row.model_code.clone(),
                    model_name: row.model_name.clone(),
                    manufacturer: row.manufacturer.clone(),
                    description: row.model_description.clone(),
                    is_active: row.model_is_active,
                    profiles: Vec::new(),
                });

                catalogue.len() - 1
            }
        };

        let model = &mut catalogue[model_index];

        let profile_index = model
            .profiles
            .iter()
            .position(|profile| profile.id == row.hardware_profile_id);

        let profile_index = match profile_index {
            Some(index) => index,
            None => {
                model.profiles.push(DeviceCatalogueHardwareProfileResponse {
                    id: row.hardware_profile_id,
                    profile_code: row.profile_code.clone(),
                    name: row.profile_name.clone(),
                    description: row.profile_description.clone(),
                    is_default: row.is_default,
                    sensors: Vec::new(),
                });

                model.profiles.len() - 1
            }
        };

        let profile = &mut model.profiles[profile_index];

        if !profile
            .sensors
            .iter()
            .any(|sensor| sensor.id == row.sensor_id)
        {
            profile.sensors.push(DeviceCatalogueSensorResponse {
                id: row.sensor_id,
                sensor_type: row.sensor_type,
                unit: row.unit,
            });
        }
    }

    Ok(catalogue)
}
