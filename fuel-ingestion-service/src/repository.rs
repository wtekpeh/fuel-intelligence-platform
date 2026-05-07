use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::FuelReading;

pub struct NewSensorReading {
    pub sensor_id: Uuid,
    pub device_id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub value: f64,
    pub unit: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub raw_payload: Value,
}

pub async fn get_or_create_demo_sensor(
    db_pool: &PgPool,
    device_code: &str,
) -> Result<(Uuid, Uuid)> {
    let organization_id = get_or_create_demo_organization(db_pool).await?;
    let asset_id = get_or_create_demo_asset(db_pool, organization_id).await?;
    let device_id = get_or_create_demo_device(db_pool, asset_id, device_code).await?;
    let sensor_id = get_or_create_fuel_sensor(db_pool, device_id).await?;

    Ok((device_id, sensor_id))
}

async fn get_or_create_demo_organization(db_pool: &PgPool) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"
        SELECT id
        FROM organizations
        WHERE name = $1
        "#,
        "Demo Transport Company"
    )
    .fetch_optional(db_pool)
    .await?
    {
        return Ok(row.id);
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO organizations (name, industry)
        VALUES ($1, $2)
        RETURNING id
        "#,
        "Demo Transport Company",
        "Transport"
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
}

async fn get_or_create_demo_asset(db_pool: &PgPool, organization_id: Uuid) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"
        SELECT id
        FROM assets
        WHERE organization_id = $1 AND name = $2
        "#,
        organization_id,
        "Demo Fuel Truck"
    )
    .fetch_optional(db_pool)
    .await?
    {
        return Ok(row.id);
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO assets (organization_id, name, asset_type, capacity_litres)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        organization_id,
        "Demo Fuel Truck",
        "truck",
        200.0_f64
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
}

async fn get_or_create_demo_device(
    db_pool: &PgPool,
    asset_id: Uuid,
    device_code: &str,
) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"
        SELECT id
        FROM devices
        WHERE device_code = $1
        "#,
        device_code
    )
    .fetch_optional(db_pool)
    .await?
    {
        return Ok(row.id);
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO devices (asset_id, device_code)
        VALUES ($1, $2)
        RETURNING id
        "#,
        asset_id,
        device_code
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
}

async fn get_or_create_fuel_sensor(db_pool: &PgPool, device_id: Uuid) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"
        SELECT id
        FROM sensors
        WHERE device_id = $1 AND sensor_code = $2
        "#,
        device_id,
        "fuel_level"
    )
    .fetch_optional(db_pool)
    .await?
    {
        return Ok(row.id);
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO sensors (device_id, sensor_code, sensor_type, unit)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        device_id,
        "fuel_level",
        "fuel_level",
        "litres"
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
}

pub async fn save_fuel_reading_as_sensor_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
    reading: &FuelReading,
) -> Result<()> {
    let raw_payload: Value = serde_json::to_value(reading)?;

    insert_sensor_reading(
        db_pool,
        NewSensorReading {
            sensor_id,
            device_id,
            recorded_at: reading.timestamp,
            value: reading.fuel_level_litres,
            unit: "litres".to_string(),
            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),
            raw_payload,
        },
    )
    .await
}

async fn insert_sensor_reading(db_pool: &PgPool, new_reading: NewSensorReading) -> Result<()> {
    sqlx::query!(
        r#"
    INSERT INTO sensor_readings (
        sensor_id,
        device_id,
        recorded_at,
        value,
        unit,
        latitude,
        longitude,
        raw_payload
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    ON CONFLICT (sensor_id, recorded_at)
    DO NOTHING
    "#,
        new_reading.sensor_id,
        new_reading.device_id,
        new_reading.recorded_at,
        new_reading.value,
        new_reading.unit,
        new_reading.latitude,
        new_reading.longitude,
        new_reading.raw_payload
    )
    .execute(db_pool)
    .await?;

    Ok(())
}
