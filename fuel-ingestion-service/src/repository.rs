use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{FuelEventResponse, FuelReading};

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

#[derive(Debug)]
pub struct StoredSensorReading {
    pub id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub value: f64,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
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

pub async fn get_previous_sensor_reading(
    db_pool: &PgPool,
    sensor_id: Uuid,
) -> Result<Option<(StoredSensorReading, StoredSensorReading)>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            recorded_at,
            value,
            latitude,
            longitude
        FROM sensor_readings
        WHERE sensor_id = $1
        ORDER BY recorded_at DESC
        LIMIT 2
        "#,
        sensor_id
    )
    .fetch_all(db_pool)
    .await?;

    if rows.len() < 2 {
        return Ok(None);
    }

    let current = StoredSensorReading {
        id: rows[0].id,
        recorded_at: rows[0].recorded_at,
        value: rows[0].value,
        latitude: rows[0].latitude,
        longitude: rows[0].longitude,
    };

    let previous = StoredSensorReading {
        id: rows[1].id,
        recorded_at: rows[1].recorded_at,
        value: rows[1].value,
        latitude: rows[1].latitude,
        longitude: rows[1].longitude,
    };

    Ok(Some((previous, current)))
}

pub async fn create_fuel_event(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
    event_type: &str,
    event_time: DateTime<Utc>,
    fuel_before: f64,
    fuel_after: f64,
    fuel_difference: f64,
    duration_seconds: i64,
    latitude: Option<f64>,
    longitude: Option<f64>,
    is_delayed_detection: bool,
    sync_delay_seconds: i64,
    severity: &str,
    message: String,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO fuel_events (
            device_id,
            sensor_id,
            event_type,
            event_time,
            fuel_before,
            fuel_after,
            fuel_difference,
            duration_seconds,
            latitude,
            longitude,
            is_delayed_detection,
            sync_delay_seconds,
            severity,
            message
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13, $14
        )
        "#,
        device_id,
        sensor_id,
        event_type,
        event_time,
        fuel_before,
        fuel_after,
        fuel_difference,
        duration_seconds,
        latitude,
        longitude,
        is_delayed_detection,
        sync_delay_seconds,
        severity,
        message
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_recent_sensor_readings(
    db_pool: &PgPool,
    sensor_id: Uuid,
    limit: i64,
) -> Result<Vec<StoredSensorReading>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            recorded_at,
            value,
            latitude,
            longitude
        FROM sensor_readings
        WHERE sensor_id = $1
        ORDER BY recorded_at DESC
        LIMIT $2
        "#,
        sensor_id,
        limit
    )
    .fetch_all(db_pool)
    .await?;

    let readings = rows
        .into_iter()
        .map(|row| StoredSensorReading {
            id: row.id,
            recorded_at: row.recorded_at,
            value: row.value,
            latitude: row.latitude,
            longitude: row.longitude,
        })
        .collect();

    Ok(readings)
}

pub async fn recent_similar_event_exists(
    db_pool: &PgPool,
    sensor_id: Uuid,
    event_type: &str,
    window_seconds: i64,
) -> Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM fuel_events
            WHERE sensor_id = $1
              AND event_type = $2
              AND detected_at >= NOW() - ($3 * INTERVAL '1 second')
        ) as "exists!"
        "#,
        sensor_id,
        event_type,
        window_seconds as f64
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.exists)
}

pub async fn recent_event_type_exists(
    db_pool: &PgPool,
    sensor_id: Uuid,
    event_type: &str,
    within_seconds: i64,
) -> Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM fuel_events
            WHERE sensor_id = $1
              AND event_type = $2
              AND event_time >= NOW() - ($3 * INTERVAL '1 second')
        ) as "exists!"
        "#,
        sensor_id,
        event_type,
        within_seconds as f64
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.exists)
}

pub async fn get_recent_fuel_events(
    db_pool: &PgPool,
    limit: i64,
) -> Result<Vec<FuelEventResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            event_type,
            event_time,
            detected_at,
            fuel_before,
            fuel_after,
            fuel_difference,
            duration_seconds,
            latitude,
            longitude,
            is_delayed_detection,
            sync_delay_seconds,
            severity,
            message
        FROM fuel_events
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| FuelEventResponse {
            id: row.id.to_string(),
            event_type: row.event_type,
            event_time: row.event_time,
            detected_at: row.detected_at,
            fuel_before: row.fuel_before,
            fuel_after: row.fuel_after,
            fuel_difference: row.fuel_difference,
            duration_seconds: row.duration_seconds,
            latitude: row.latitude,
            longitude: row.longitude,
            is_delayed_detection: row.is_delayed_detection,
            sync_delay_seconds: row.sync_delay_seconds,
            severity: row.severity,
            message: row.message,
        })
        .collect();

    Ok(events)
}
