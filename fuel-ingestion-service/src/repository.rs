use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    DeviceHealthEventResponse, DeviceStateEventResponse, FuelEventResponse, FuelReading,
    SensorHealthEventResponse,
};
use crate::services::device_health::classify_device_status;
use crate::services::device_state::{
    calculate_distance_meters, calculate_speed_kmh, classify_device_state,
};

pub struct NewSensorReading {
    pub sensor_id: Uuid,
    pub device_id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub value: f64,
    pub unit: String,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub vibration_level: Option<f64>,
    pub motion_detected: Option<bool>,

    pub raw_payload: Value,
}

pub struct NewDeviceStateEvent {
    pub device_id: Uuid,
    pub sensor_id: Option<Uuid>,

    pub state: String,

    pub recorded_at: DateTime<Utc>,

    pub vibration_level: Option<f64>,
    pub motion_detected: Option<bool>,

    pub distance_meters: Option<f64>,
    pub speed_kmh: Option<f64>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub source: String,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct StoredSensorReading {
    pub id: Uuid,
    pub recorded_at: DateTime<Utc>,
    pub value: f64,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub struct PreviousLocationReading {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

pub async fn get_latest_device_state(db_pool: &PgPool, device_id: Uuid) -> Result<Option<String>> {
    let row = sqlx::query!(
        r#"
        SELECT state
        FROM device_state_events
        WHERE device_id = $1
        ORDER BY recorded_at DESC
        LIMIT 1
        "#,
        device_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.map(|row| row.state))
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
    previous_reading: Option<&FuelReading>,
) -> Result<()> {
    let (previous_latitude, previous_longitude, distance_meters, speed_kmh) = match previous_reading
    {
        Some(prev) => {
            let distance_meters = calculate_distance_meters(
                prev.latitude,
                prev.longitude,
                reading.latitude,
                reading.longitude,
            );

            let time_seconds = (reading.timestamp - prev.timestamp).num_seconds().max(0) as f64;

            let speed_kmh = calculate_speed_kmh(distance_meters, time_seconds);

            (
                Some(prev.latitude),
                Some(prev.longitude),
                Some(distance_meters),
                Some(speed_kmh),
            )
        }
        None => (None, None, None, None),
    };

    let device_state = classify_device_state(
        Some("ONLINE"),
        Some(reading.vibration_level),
        Some(reading.motion_detected),
        previous_latitude,
        previous_longitude,
        Some(reading.latitude),
        Some(reading.longitude),
    );

    create_device_state_event(
        db_pool,
        NewDeviceStateEvent {
            device_id,
            sensor_id: Some(sensor_id),

            state: device_state.as_str().to_string(),

            recorded_at: reading.timestamp,

            vibration_level: Some(reading.vibration_level),
            motion_detected: Some(reading.motion_detected),

            distance_meters,
            speed_kmh,

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            source: "telemetry".to_string(),

            message: Some(format!(
                "State classified from telemetry: {:?}",
                device_state
            )),
        },
    )
    .await?;

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

            vibration_level: Some(reading.vibration_level),
            motion_detected: Some(reading.motion_detected),

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

        vibration_level,
        motion_detected,

        raw_payload
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        new_reading.vibration_level,
        new_reading.motion_detected,
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

pub async fn mark_device_payload_seen(db_pool: &PgPool, device_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE devices
        SET
            status = 'ONLINE',
            last_seen_at = NOW(),
            last_payload_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        device_id
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn mark_device_heartbeat_seen(db_pool: &PgPool, device_code: &str) -> Result<Uuid> {
    let (device_id, _sensor_id) = get_or_create_demo_sensor(db_pool, device_code).await?;

    let device = sqlx::query!(
        r#"
        SELECT
            status,
            last_seen_at,
            last_heartbeat_at,
            last_payload_at
        FROM devices
        WHERE id = $1
        "#,
        device_id
    )
    .fetch_one(db_pool)
    .await?;

    sqlx::query!(
        r#"
        UPDATE devices
        SET
            status = 'ONLINE',
            last_seen_at = NOW(),
            last_heartbeat_at = NOW(),
            updated_at = NOW()
        WHERE id = $1
        "#,
        device_id
    )
    .execute(db_pool)
    .await?;

    if device.status != "ONLINE" {
        sqlx::query!(
            r#"
            INSERT INTO device_health_events (
                device_id,
                previous_status,
                new_status,
                reason,
                last_seen_at,
                last_heartbeat_at,
                last_payload_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            device_id,
            device.status,
            "ONLINE",
            "heartbeat_received",
            device.last_seen_at,
            device.last_heartbeat_at,
            device.last_payload_at
        )
        .execute(db_pool)
        .await?;
    }

    Ok(device_id)
}

pub async fn refresh_device_statuses(
    db_pool: &PgPool,
    stale_after_seconds: i64,
    offline_after_seconds: i64,
) -> Result<()> {
    let devices = sqlx::query!(
        r#"
    SELECT
        id,
        status,
        last_seen_at,
        last_heartbeat_at,
        last_payload_at
    FROM devices
    "#
    )
    .fetch_all(db_pool)
    .await?;

    for device in devices {
        let new_status = classify_device_status(
            device.last_seen_at,
            stale_after_seconds,
            offline_after_seconds,
        );

        if new_status == device.status {
            continue;
        }

        sqlx::query!(
            r#"
        UPDATE devices
        SET
            status = $1,
            updated_at = NOW()
        WHERE id = $2
        "#,
            new_status,
            device.id
        )
        .execute(db_pool)
        .await?;

        sqlx::query!(
            r#"
        INSERT INTO device_health_events (
            device_id,
            previous_status,
            new_status,
            reason,
            last_seen_at,
            last_heartbeat_at,
            last_payload_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
            device.id,
            device.status,
            new_status,
            "status_changed_by_health_refresh",
            device.last_seen_at,
            device.last_heartbeat_at,
            device.last_payload_at
        )
        .execute(db_pool)
        .await?;
    }

    Ok(())
}

pub async fn get_recent_device_health_events(
    db_pool: &PgPool,
    limit: i64,
) -> Result<Vec<DeviceHealthEventResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            previous_status,
            new_status,
            reason,
            detected_at
        FROM device_health_events
        ORDER BY detected_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| DeviceHealthEventResponse {
            id: row.id.to_string(),
            device_id: row.device_id.to_string(),
            previous_status: row.previous_status,
            new_status: row.new_status,
            reason: row.reason,
            detected_at: row.detected_at,
        })
        .collect();

    Ok(events)
}

pub async fn create_sensor_health_event(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
    event_type: &str,
    severity: &str,
    reason: &str,
    first_seen_at: Option<DateTime<Utc>>,
    last_seen_at: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO sensor_health_events (
            device_id,
            sensor_id,
            event_type,
            severity,
            reason,
            first_seen_at,
            last_seen_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        device_id,
        sensor_id,
        event_type,
        severity,
        reason,
        first_seen_at,
        last_seen_at
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn recent_sensor_health_event_exists(
    db_pool: &PgPool,
    sensor_id: Uuid,
    event_type: &str,
    window_seconds: i64,
) -> Result<bool> {
    let row = sqlx::query!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM sensor_health_events
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

pub async fn get_recent_sensor_health_events(
    db_pool: &PgPool,
    limit: i64,
) -> Result<Vec<SensorHealthEventResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            device_id,
            sensor_id,
            event_type,
            severity,
            reason,
            first_seen_at,
            last_seen_at,
            detected_at
        FROM sensor_health_events
        ORDER BY detected_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| SensorHealthEventResponse {
            id: row.id.to_string(),
            device_id: row.device_id.to_string(),
            sensor_id: row.sensor_id.to_string(),
            event_type: row.event_type,
            severity: row.severity,
            reason: row.reason,
            first_seen_at: row.first_seen_at,
            last_seen_at: row.last_seen_at,
            detected_at: row.detected_at,
        })
        .collect();

    Ok(events)
}

pub async fn create_device_state_event(
    db_pool: &PgPool,
    new_event: NewDeviceStateEvent,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO device_state_events (
            device_id,
            sensor_id,
            state,
            recorded_at,
            vibration_level,
            motion_detected,
            distance_meters,
            speed_kmh,
            latitude,
            longitude,
            source,
            message
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12
        )
        "#,
        new_event.device_id,
        new_event.sensor_id,
        new_event.state,
        new_event.recorded_at,
        new_event.vibration_level,
        new_event.motion_detected,
        new_event.distance_meters,
        new_event.speed_kmh,
        new_event.latitude,
        new_event.longitude,
        new_event.source,
        new_event.message
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn get_recent_device_state_events(
    db_pool: &PgPool,
    limit: i64,
) -> Result<Vec<DeviceStateEventResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            state,
            vibration_level,
            motion_detected,
            latitude,
            longitude,
            recorded_at
        FROM device_state_events
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(db_pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| DeviceStateEventResponse {
            state: row.state,
            vibration_level: row.vibration_level,
            motion_detected: row.motion_detected,
            latitude: row.latitude,
            longitude: row.longitude,
            recorded_at: row.recorded_at,
        })
        .collect();

    Ok(events)
}

async fn get_previous_location_reading(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_id: Uuid,
    recorded_at: DateTime<Utc>,
) -> Result<Option<PreviousLocationReading>> {
    let row = sqlx::query!(
        r#"
        SELECT
            latitude,
            longitude,
            recorded_at
        FROM sensor_readings
        WHERE device_id = $1
            AND sensor_id = $2
            AND recorded_at < $3
        ORDER BY recorded_at DESC
        LIMIT 1
        "#,
        device_id,
        sensor_id,
        recorded_at
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.map(|row| PreviousLocationReading {
        latitude: row.latitude,
        longitude: row.longitude,
        recorded_at: row.recorded_at,
    }))
}
