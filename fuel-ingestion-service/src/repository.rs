use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AlertAcknowledgementResponse, AlertResponse, AlertTrendPoint, AlertTrendSummary,
    AlertTrendsResponse, AssignDeviceAssetRequest, CreateAssetRequest, CreateGeofenceRequest,
    CreateOrganizationRequest, DeviceHealthEventResponse, DeviceHealthTrendDevice,
    DeviceHealthTrendResponse, DeviceModelResponse, DeviceSensorSummary, DeviceStateEventResponse,
    DeviceSummary, FuelEventResponse, FuelReading, Geofence, GeofenceActivityTrendPoint,
    GeofenceActivityTrendResponse, GeofencePositionMatch, GeofenceTransitionEventResponse,
    GeofenceUtilizationResponse, GeofenceUtilizationZone, HardwareProfile, HardwareProfileSensor,
    OrganizationFleetOverviewResponse, OrganizationOverviewResponse, SensorHealthEventResponse,
    TelemetryStreamResponse, UpdateAssetRequest, UpdateDeviceRequest, UpdateOrganizationRequest,
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

#[derive(Debug)]
pub struct RegisteredDeviceContext {
    pub device_id: Uuid,
    pub fuel_sensor_id: Uuid,
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

// -----------------------------------------------------------------------------
// Platform Management
// -----------------------------------------------------------------------------
pub async fn find_registered_device_context(
    db_pool: &PgPool,
    device_code: &str,
) -> Result<Option<RegisteredDeviceContext>> {
    let row = sqlx::query!(
        r#"
        SELECT
            d.id AS device_id,
            s.id AS fuel_sensor_id
        FROM devices d
        INNER JOIN sensors s
            ON s.device_id = d.id
        WHERE d.device_code = $1
          AND s.sensor_type = 'FUEL'
        LIMIT 1
        "#,
        device_code
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.map(|row| RegisteredDeviceContext {
        device_id: row.device_id,
        fuel_sensor_id: row.fuel_sensor_id,
    }))
}

pub async fn get_hardware_profile_sensors(
    db_pool: &PgPool,
    hardware_profile_id: Uuid,
) -> Result<Vec<HardwareProfileSensor>> {
    let sensors = sqlx::query_as!(
        HardwareProfileSensor,
        r#"
        SELECT
            id,
            hardware_profile_id,
            sensor_type,
            unit,
            created_at
        FROM hardware_profile_sensors
        WHERE hardware_profile_id = $1
        ORDER BY sensor_type
        "#,
        hardware_profile_id
    )
    .fetch_all(db_pool)
    .await?;

    Ok(sensors)
}

pub async fn create_sensors_for_hardware_profile(
    db_pool: &PgPool,
    device_id: Uuid,
    hardware_profile_id: Uuid,
) -> Result<Vec<Uuid>> {
    let profile_sensors = get_hardware_profile_sensors(db_pool, hardware_profile_id).await?;

    let mut created_sensor_ids = Vec::new();

    for profile_sensor in profile_sensors {
        let sensor_code = profile_sensor.sensor_type.to_lowercase();

        let sensor_id = get_or_create_sensor(
            db_pool,
            device_id,
            &sensor_code,
            &profile_sensor.sensor_type,
            &profile_sensor.unit,
        )
        .await?;

        created_sensor_ids.push(sensor_id);
    }

    Ok(created_sensor_ids)
}

pub async fn register_device(
    db_pool: &PgPool,
    asset_id: Uuid,
    device_model_id: Option<Uuid>,
    device_code: String,
    hardware_profile_id: Uuid,
) -> Result<Uuid> {
    let device_id = Uuid::new_v4();

    let row = sqlx::query!(
        r#"
        INSERT INTO devices (
            id,
            asset_id,
            device_model_id,
            device_code,
            hardware_profile_id
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        device_id,
        asset_id,
        device_model_id,
        device_code,
        hardware_profile_id,
    )
    .fetch_one(db_pool)
    .await?;

    create_sensors_for_hardware_profile(db_pool, row.id, hardware_profile_id).await?;

    Ok(row.id)
}

pub async fn list_hardware_profiles(db_pool: &PgPool) -> Result<Vec<HardwareProfile>> {
    let profiles = sqlx::query_as!(
        HardwareProfile,
        r#"
        SELECT
            id,
            profile_code,
            name,
            description,
            created_at
        FROM hardware_profiles
        ORDER BY profile_code
        "#
    )
    .fetch_all(db_pool)
    .await?;

    Ok(profiles)
}

pub async fn list_device_models(db_pool: &PgPool) -> Result<Vec<DeviceModelResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            id,
            model_code,
            model_name,
            manufacturer,
            description,
            is_active,
            created_at
        FROM device_models
        WHERE is_active = TRUE
        ORDER BY model_name
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let models = rows
        .into_iter()
        .map(|row| DeviceModelResponse {
            id: row.id,
            model_code: row.model_code,
            model_name: row.model_name,
            manufacturer: row.manufacturer,
            description: row.description,
            is_active: row.is_active,
            created_at: row.created_at,
        })
        .collect();

    Ok(models)
}

pub async fn list_devices(db_pool: &PgPool) -> Result<Vec<DeviceSummary>> {
    let devices = sqlx::query_as!(
        DeviceSummary,
        r#"
        SELECT
            d.id,
            d.device_code,
            d.asset_id,

            d.device_model_id,
            dm.model_code AS "device_model_code?",
            dm.model_name AS "device_model_name?",

            d.hardware_profile_id,
            hp.profile_code AS hardware_profile_code,
            hp.name AS hardware_profile_name,

            d.status,
            d.created_at

        FROM devices d

        LEFT JOIN device_models dm
            ON dm.id = d.device_model_id

        INNER JOIN hardware_profiles hp
            ON hp.id = d.hardware_profile_id

        ORDER BY d.created_at DESC
        "#
    )
    .fetch_all(db_pool)
    .await?;

    Ok(devices)
}

pub async fn list_device_sensors(
    db_pool: &PgPool,
    device_id: Uuid,
) -> Result<Vec<DeviceSensorSummary>> {
    let sensors = sqlx::query_as!(
        DeviceSensorSummary,
        r#"
        SELECT
            id,
            device_id,
            sensor_code,
            sensor_type,
            unit,
            created_at
        FROM sensors
        WHERE device_id = $1
        ORDER BY sensor_type
        "#,
        device_id
    )
    .fetch_all(db_pool)
    .await?;

    Ok(sensors)
}

// -----------------------------------------------------------------------------
// Fuel Simulator Bootstrap
// -----------------------------------------------------------------------------

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

async fn get_or_create_sensor(
    db_pool: &PgPool,
    device_id: Uuid,
    sensor_code: &str,
    sensor_type: &str,
    unit: &str,
) -> Result<Uuid> {
    if let Some(row) = sqlx::query!(
        r#"
        SELECT id
        FROM sensors
        WHERE device_id = $1 AND sensor_code = $2
        "#,
        device_id,
        sensor_code
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
        sensor_code,
        sensor_type,
        unit
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
}

async fn get_or_create_fuel_sensor(db_pool: &PgPool, device_id: Uuid) -> Result<Uuid> {
    get_or_create_sensor(db_pool, device_id, "fuel_level", "fuel_level", "litres").await
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
    confidence: Option<String>,
    correlation_status: Option<String>,
    correlation_reason: Option<String>,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
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
            message,
            confidence,
            correlation_status,
            correlation_reason
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13, $14, $15, $16, $17
        )
        RETURNING id
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
        message,
        confidence,
        correlation_status,
        correlation_reason
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.id)
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
    device_id: Option<Uuid>,
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
            confidence,
            correlation_status,
            correlation_reason,
            message
        FROM fuel_events
        WHERE
            $2::uuid IS NULL
            OR device_id = $2
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit,
        device_id
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
            confidence: row.confidence,
            correlation_status: row.correlation_status,
            correlation_reason: row.correlation_reason,
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
    let context = find_registered_device_context(db_pool, device_code).await?;

    let Some(context) = context else {
        anyhow::bail!(
            "Unknown device '{}'. Device must be provisioned before heartbeats can be accepted.",
            device_code
        );
    };

    let device_id = context.device_id;

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
    device_id: Option<Uuid>,
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
        WHERE
            $2::uuid IS NULL
            OR device_id = $2
        ORDER BY detected_at DESC
        LIMIT $1
        "#,
        limit,
        device_id
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
    device_id: Option<Uuid>,
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
        WHERE
            $2::uuid IS NULL
            OR device_id = $2
        ORDER BY detected_at DESC
        LIMIT $1
        "#,
        limit,
        device_id
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
    device_id: Option<Uuid>,
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
        WHERE
            $2::uuid IS NULL
            OR device_id = $2
        ORDER BY created_at DESC
        LIMIT $1
        "#,
        limit,
        device_id
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

pub async fn create_alert(
    db_pool: &PgPool,
    fuel_event_id: Option<Uuid>,
    alert_type: String,
    severity: String,
    reason: String,
) -> Result<AlertResponse, sqlx::Error> {
    let alert = sqlx::query_as!(
        AlertResponse,
        r#"
        WITH inserted_alert AS (
            INSERT INTO alerts (
                fuel_event_id,
                alert_type,
                severity,
                reason
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                fuel_event_id,
                alert_type,
                severity,
                reason,
                is_acknowledged,
                status,
                created_at
        )
        SELECT
            inserted_alert.id AS "id!",
            inserted_alert.fuel_event_id,
            fuel_events.device_id,
            inserted_alert.alert_type AS "alert_type!",
            inserted_alert.severity AS "severity!",
            inserted_alert.reason AS "reason!",
            inserted_alert.is_acknowledged AS "is_acknowledged!",
            inserted_alert.status AS "status!",
            inserted_alert.created_at AS "created_at!"
        FROM inserted_alert
        LEFT JOIN fuel_events
            ON fuel_events.id = inserted_alert.fuel_event_id
        "#,
        fuel_event_id,
        alert_type,
        severity,
        reason
    )
    .fetch_one(db_pool)
    .await?;

    Ok(alert)
}

pub async fn get_recent_alerts(
    db_pool: &PgPool,
    device_id: Option<Uuid>,
) -> Result<Vec<AlertResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
    SELECT
        alerts.id,
        alerts.fuel_event_id,
        fuel_events.device_id,
        alerts.alert_type,
        alerts.severity,
        alerts.reason,
        alerts.is_acknowledged,
        alerts.status,
        alerts.created_at
    FROM alerts
    LEFT JOIN fuel_events
        ON fuel_events.id = alerts.fuel_event_id
    WHERE
        $1::uuid IS NULL
        OR fuel_events.device_id = $1
    ORDER BY alerts.created_at DESC
    LIMIT 100
    "#,
        device_id
    )
    .fetch_all(db_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| AlertResponse {
            id: row.id,
            fuel_event_id: row.fuel_event_id,
            device_id: Some(row.device_id),
            alert_type: row.alert_type,
            severity: row.severity,
            reason: row.reason,
            is_acknowledged: row.is_acknowledged,
            status: row.status,
            created_at: row.created_at,
        })
        .collect())
}

pub async fn get_alert_trends(
    db_pool: &PgPool,
    device_id: Option<Uuid>,
    days: i64,
) -> Result<AlertTrendsResponse, sqlx::Error> {
    let safe_days = if days < 1 {
        30
    } else if days > 90 {
        90
    } else {
        days
    };

    let summary_row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) AS "total_alerts!",
            COUNT(*) FILTER (WHERE alerts.alert_type = 'THEFT') AS "theft_alerts!",
            COUNT(*) FILTER (WHERE alerts.alert_type = 'REFILL') AS "refill_alerts!",
            COUNT(*) FILTER (WHERE alerts.alert_type = 'LEAK') AS "leak_alerts!",
            COUNT(*) FILTER (WHERE alerts.status = 'OPEN') AS "open_alerts!",
            COUNT(*) FILTER (WHERE alerts.status = 'ACKNOWLEDGED') AS "acknowledged_alerts!",
            COUNT(*) FILTER (WHERE alerts.status = 'RESOLVED') AS "resolved_alerts!"
        FROM alerts
        LEFT JOIN fuel_events
            ON fuel_events.id = alerts.fuel_event_id
        WHERE
            alerts.created_at >= NOW() - ($2 * INTERVAL '1 day')
            AND (
                $1::uuid IS NULL
                OR fuel_events.device_id = $1
            )
        "#,
        device_id,
        safe_days as f64
    )
    .fetch_one(db_pool)
    .await?;

    let summary = AlertTrendSummary {
        total_alerts: summary_row.total_alerts,
        theft_alerts: summary_row.theft_alerts,
        refill_alerts: summary_row.refill_alerts,
        leak_alerts: summary_row.leak_alerts,
        open_alerts: summary_row.open_alerts,
        acknowledged_alerts: summary_row.acknowledged_alerts,
        resolved_alerts: summary_row.resolved_alerts,
    };

    let trend_rows = sqlx::query!(
        r#"
    SELECT
        alerts.created_at::date AS "day!",
        alerts.alert_type AS "alert_type!",
        alerts.status AS "status!",
        COUNT(*) AS "count!"
    FROM alerts
    LEFT JOIN fuel_events
        ON fuel_events.id = alerts.fuel_event_id
    WHERE
        alerts.created_at >= NOW() - ($2 * INTERVAL '1 day')
        AND (
            $1::uuid IS NULL
            OR fuel_events.device_id = $1
        )
    GROUP BY
        alerts.created_at::date,
        alerts.alert_type,
        alerts.status
    ORDER BY
        alerts.created_at::date ASC,
        alerts.alert_type ASC,
        alerts.status ASC
    "#,
        device_id,
        safe_days as f64
    )
    .fetch_all(db_pool)
    .await?;

    let trend = trend_rows
        .into_iter()
        .map(|row| AlertTrendPoint {
            day: row.day.to_string(),
            alert_type: row.alert_type,
            status: row.status,
            count: row.count,
        })
        .collect();

    Ok(AlertTrendsResponse {
        days: safe_days,
        summary,
        trend,
    })
}

pub async fn list_alerts_since(
    db_pool: &PgPool,
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<AlertResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            alerts.id,
            alerts.fuel_event_id,
            fuel_events.device_id,
            alerts.alert_type,
            alerts.severity,
            alerts.reason,
            alerts.is_acknowledged,
            alerts.status,
            alerts.created_at
        FROM alerts
        LEFT JOIN fuel_events
            ON fuel_events.id = alerts.fuel_event_id
        WHERE alerts.created_at > $1
        ORDER BY alerts.created_at ASC
        "#,
        since
    )
    .fetch_all(db_pool)
    .await?;

    let alerts = rows
        .into_iter()
        .map(|row| AlertResponse {
            id: row.id,
            fuel_event_id: row.fuel_event_id,
            device_id: Some(row.device_id),
            alert_type: row.alert_type,
            severity: row.severity,
            reason: row.reason,
            is_acknowledged: row.is_acknowledged,
            status: row.status,
            created_at: row.created_at,
        })
        .collect();

    Ok(alerts)
}

pub async fn acknowledge_alert(
    db_pool: &PgPool,
    alert_id: Uuid,
) -> Result<Option<AlertAcknowledgementResponse>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE alerts
        SET
            is_acknowledged = true,
            status = 'ACKNOWLEDGED'
        WHERE id = $1
        RETURNING
            id,
            alert_type,
            severity,
            is_acknowledged,
            status,
            created_at
        "#,
        alert_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.map(|row| AlertAcknowledgementResponse {
        id: row.id,
        alert_type: row.alert_type,
        severity: row.severity,
        is_acknowledged: row.is_acknowledged,
        status: row.status,
        created_at: row.created_at,
    }))
}

pub async fn resolve_alert(
    db_pool: &PgPool,
    alert_id: Uuid,
) -> Result<Option<AlertAcknowledgementResponse>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE alerts
        SET
            is_acknowledged = true,
            status = 'RESOLVED'
        WHERE id = $1
        RETURNING
            id,
            alert_type,
            severity,
            is_acknowledged,
            status,
            created_at
        "#,
        alert_id
    )
    .fetch_optional(db_pool)
    .await?;

    Ok(row.map(|row| AlertAcknowledgementResponse {
        id: row.id,
        alert_type: row.alert_type,
        severity: row.severity,
        is_acknowledged: row.is_acknowledged,
        status: row.status,
        created_at: row.created_at,
    }))
}

pub async fn get_recent_telemetry_stream(
    db_pool: &PgPool,
    device_id: Option<Uuid>,
) -> Result<Vec<TelemetryStreamResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            device_id,
            value AS fuel_level_litres,
            latitude,
            longitude,
            vibration_level,
            motion_detected,
            recorded_at,
            received_at
        FROM sensor_readings
        WHERE
            $1::uuid IS NULL
            OR device_id = $1
        ORDER BY received_at DESC
        LIMIT 10
        "#,
        device_id
    )
    .fetch_all(db_pool)
    .await?;

    let readings = rows
        .into_iter()
        .map(|row| TelemetryStreamResponse {
            device_id: row.device_id.to_string(),
            fuel_level_litres: row.fuel_level_litres,
            latitude: row.latitude,
            longitude: row.longitude,
            vibration_level: row.vibration_level,
            motion_detected: row.motion_detected,
            recorded_at: row.recorded_at,
            received_at: row.received_at,
        })
        .collect();

    Ok(readings)
}

pub async fn get_organization_overview(
    db_pool: &PgPool,
) -> Result<Vec<OrganizationOverviewResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            organizations.id AS organization_id,
            organizations.name AS organization_name,
            organizations.industry AS industry,

            COUNT(DISTINCT assets.id) AS asset_count,
            COUNT(DISTINCT devices.id) AS device_count,

            COUNT(DISTINCT devices.id) FILTER (
                WHERE devices.status = 'ONLINE'
            ) AS online_device_count,

            COUNT(DISTINCT devices.id) FILTER (
                WHERE devices.status = 'STALE'
            ) AS stale_device_count,

            COUNT(DISTINCT devices.id) FILTER (
                WHERE devices.status = 'OFFLINE'
            ) AS offline_device_count,

            COUNT(DISTINCT alerts.id) FILTER (
                WHERE alerts.status = 'OPEN'
            ) AS open_alert_count

        FROM organizations
        LEFT JOIN assets
            ON assets.organization_id = organizations.id
        LEFT JOIN devices
            ON devices.asset_id = assets.id
        LEFT JOIN fuel_events
            ON fuel_events.device_id = devices.id
        LEFT JOIN alerts
            ON alerts.fuel_event_id = fuel_events.id

        GROUP BY
            organizations.id,
            organizations.name,
            organizations.industry

        ORDER BY organizations.name ASC
        "#
    )
    .fetch_all(db_pool)
    .await?;

    let overview = rows
        .into_iter()
        .map(|row| OrganizationOverviewResponse {
            organization_id: row.organization_id,
            organization_name: row.organization_name,
            industry: row.industry,
            asset_count: row.asset_count.unwrap_or(0),
            device_count: row.device_count.unwrap_or(0),
            online_device_count: row.online_device_count.unwrap_or(0),
            stale_device_count: row.stale_device_count.unwrap_or(0),
            offline_device_count: row.offline_device_count.unwrap_or(0),
            open_alert_count: row.open_alert_count.unwrap_or(0),
        })
        .collect();

    Ok(overview)
}

pub async fn get_organization_fleet_overview(
    db_pool: &PgPool,
    organization_id: Uuid,
) -> Result<Vec<OrganizationFleetOverviewResponse>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            assets.id AS asset_id,
            assets.name AS asset_name,
            assets.asset_type AS asset_type,
            assets.capacity_litres AS capacity_litres,

            devices.id AS "device_id?",
            devices.device_code AS "device_code?",
            devices.status AS "device_status?",
            devices.last_seen_at AS last_seen_at,

            COUNT(DISTINCT sensors.id) AS sensor_count,

            COALESCE(
                ARRAY_AGG(DISTINCT sensors.sensor_type)
                FILTER (WHERE sensors.sensor_type IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS sensor_types,

            COUNT(DISTINCT alerts.id) FILTER (
                WHERE alerts.status = 'OPEN'
            ) AS open_alert_count

        FROM assets

        LEFT JOIN devices
            ON devices.asset_id = assets.id

        LEFT JOIN sensors
            ON sensors.device_id = devices.id

        LEFT JOIN fuel_events
            ON fuel_events.device_id = devices.id

        LEFT JOIN alerts
            ON alerts.fuel_event_id = fuel_events.id

        WHERE assets.organization_id = $1

        GROUP BY
            assets.id,
            assets.name,
            assets.asset_type,
            assets.capacity_litres,
            devices.id,
            devices.device_code,
            devices.status,
            devices.last_seen_at

        ORDER BY
            assets.name ASC,
            devices.device_code ASC
        "#,
        organization_id
    )
    .fetch_all(db_pool)
    .await?;

    let overview = rows
        .into_iter()
        .map(|row| OrganizationFleetOverviewResponse {
            asset_id: row.asset_id,
            asset_name: row.asset_name,
            asset_type: row.asset_type,
            capacity_litres: row.capacity_litres,

            device_id: row.device_id,
            device_code: row.device_code,
            device_status: row.device_status,
            last_seen_at: row.last_seen_at,

            sensor_count: row.sensor_count.unwrap_or(0),

            sensor_types: row.sensor_types.unwrap_or_default(),

            open_alert_count: row.open_alert_count.unwrap_or(0),
        })
        .collect();

    Ok(overview)
}

pub async fn create_organization(
    db_pool: &PgPool,
    request: &CreateOrganizationRequest,
) -> Result<Uuid> {
    let organization_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO organizations (
            id,
            name,
            industry
        )
        VALUES ($1, $2, $3)
        "#,
        organization_id,
        request.organization_name,
        request.industry,
    )
    .execute(db_pool)
    .await?;

    Ok(organization_id)
}

pub async fn update_organization(
    db_pool: &PgPool,
    organization_id: Uuid,
    request: &UpdateOrganizationRequest,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE organizations
        SET
            name = $2,
            industry = $3
        WHERE id = $1
        "#,
        organization_id,
        request.organization_name,
        request.industry,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn archive_organization(db_pool: &PgPool, organization_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE organizations
        SET is_active = FALSE
        WHERE id = $1
        "#,
        organization_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn create_asset(db_pool: &PgPool, request: &CreateAssetRequest) -> Result<Uuid> {
    let asset_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO assets (
            id,
            organization_id,
            name,
            asset_type,
            metadata,
            is_active
        )
        VALUES ($1, $2, $3, $4, $5, TRUE)
        "#,
        asset_id,
        request.organization_id,
        request.name,
        request.asset_type,
        request.metadata,
    )
    .execute(db_pool)
    .await?;

    Ok(asset_id)
}

pub async fn update_asset(
    db_pool: &PgPool,
    asset_id: Uuid,
    request: &UpdateAssetRequest,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE assets
        SET
            name = $2,
            asset_type = $3,
            metadata = $4
        WHERE id = $1
        "#,
        asset_id,
        request.name,
        request.asset_type,
        request.metadata,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn archive_asset(db_pool: &PgPool, asset_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE assets
        SET is_active = FALSE
        WHERE id = $1
        "#,
        asset_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn update_device(
    db_pool: &PgPool,
    device_id: Uuid,
    request: &UpdateDeviceRequest,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE devices
        SET
            device_code = $2,
            hardware_profile_id = $3
        WHERE id = $1
        "#,
        device_id,
        request.device_code,
        request.hardware_profile_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn deactivate_device(db_pool: &PgPool, device_id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE devices
        SET is_active = FALSE
        WHERE id = $1
        "#,
        device_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn assign_device_to_asset(
    db_pool: &PgPool,
    device_id: Uuid,
    request: &AssignDeviceAssetRequest,
) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE devices
        SET asset_id = $2
        WHERE id = $1
        "#,
        device_id,
        request.asset_id,
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn create_geofence(
    pool: &sqlx::PgPool,
    payload: CreateGeofenceRequest,
) -> Result<Geofence, sqlx::Error> {
    let geofence = sqlx::query_as::<_, Geofence>(
        r#"
        INSERT INTO geofences (
            id,
            organization_id,
            name,
            geofence_type,
            geometry,
            is_active
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            ST_SetSRID(
                ST_GeomFromGeoJSON($5),
                4326
            ),
            TRUE
        )
        RETURNING
            id,
            organization_id,
            name,
            geofence_type,
            ST_AsGeoJSON(geometry)::json AS geojson,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(payload.organization_id)
    .bind(payload.name)
    .bind(payload.geofence_type)
    .bind(payload.geojson.to_string())
    .fetch_one(pool)
    .await?;

    Ok(geofence)
}

pub async fn list_geofences(
    pool: &sqlx::PgPool,
    organization_id: uuid::Uuid,
) -> Result<Vec<Geofence>, sqlx::Error> {
    let geofences = sqlx::query_as::<_, Geofence>(
        r#"
        SELECT
            id,
            organization_id,
            name,
            geofence_type,
            ST_AsGeoJSON(geometry)::json AS geojson,
            is_active,
            created_at,
            updated_at
        FROM geofences
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(organization_id)
    .fetch_all(pool)
    .await?;

    Ok(geofences)
}

async fn get_matching_geofences_for_position(
    pool: &PgPool,
    organization_id: uuid::Uuid,
    device_id: uuid::Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<GeofencePositionMatch>, sqlx::Error> {
    let matches = sqlx::query_as!(
        GeofencePositionMatch,
        r#"
        SELECT
            g.id AS geofence_id,
            g.name AS geofence_name,
            g.geofence_type
        FROM geofences g
        WHERE
            g.organization_id = $1
            AND g.is_active = TRUE

            AND ST_Contains(
                g.geometry,
                ST_SetSRID(
                    ST_MakePoint($2, $3),
                    4326
                )
            )

            AND (
                NOT EXISTS (
                    SELECT 1
                    FROM geofence_device_assignments gda
                    WHERE gda.geofence_id = g.id
                )

                OR EXISTS (
                    SELECT 1
                    FROM geofence_device_assignments gda
                    WHERE
                        gda.geofence_id = g.id
                        AND gda.device_id = $4
                        AND gda.is_included = TRUE
                )
            )
        "#,
        organization_id,
        longitude,
        latitude,
        device_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(matches)
}

pub async fn get_organization_id_for_device(
    pool: &PgPool,
    device_id: uuid::Uuid,
) -> Result<uuid::Uuid, sqlx::Error> {
    let record = sqlx::query!(
        r#"
        SELECT
            a.organization_id
        FROM devices d
        JOIN assets a
            ON a.id = d.asset_id
        WHERE d.id = $1
        "#,
        device_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(record.organization_id)
}

pub async fn check_position_against_geofences(
    pool: &PgPool,
    organization_id: Uuid,
    device_id: Uuid,
    latitude: f64,
    longitude: f64,
) -> Result<Vec<GeofencePositionMatch>, sqlx::Error> {
    let matches =
        get_matching_geofences_for_position(pool, organization_id, device_id, latitude, longitude)
            .await?;

    Ok(matches)
}

async fn insert_geofence_transition_event(
    pool: &PgPool,
    organization_id: uuid::Uuid,
    device_id: uuid::Uuid,
    geofence_id: uuid::Uuid,
    transition_type: &str,
    latitude: f64,
    longitude: f64,
    recorded_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO geofence_transition_events (
            id,
            organization_id,
            device_id,
            geofence_id,
            transition_type,
            latitude,
            longitude,
            recorded_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        uuid::Uuid::new_v4(),
        organization_id,
        device_id,
        geofence_id,
        transition_type,
        latitude,
        longitude,
        recorded_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn detect_and_store_geofence_transitions_from_previous_position(
    pool: &PgPool,
    organization_id: uuid::Uuid,
    device_id: uuid::Uuid,
    previous_latitude: f64,
    previous_longitude: f64,
    current_latitude: f64,
    current_longitude: f64,
    recorded_at: chrono::DateTime<chrono::Utc>,
) -> Result<(), sqlx::Error> {
    let previous_matches = get_matching_geofences_for_position(
        pool,
        organization_id,
        device_id,
        previous_latitude,
        previous_longitude,
    )
    .await?;

    let current_matches = get_matching_geofences_for_position(
        pool,
        organization_id,
        device_id,
        current_latitude,
        current_longitude,
    )
    .await?;

    let previous_ids: std::collections::HashSet<uuid::Uuid> = previous_matches
        .iter()
        .map(|geofence| geofence.geofence_id)
        .collect();

    let current_ids: std::collections::HashSet<uuid::Uuid> = current_matches
        .iter()
        .map(|geofence| geofence.geofence_id)
        .collect();

    for geofence in &current_matches {
        if !previous_ids.contains(&geofence.geofence_id) {
            insert_geofence_transition_event(
                pool,
                organization_id,
                device_id,
                geofence.geofence_id,
                "ENTERED_ZONE",
                current_latitude,
                current_longitude,
                recorded_at,
            )
            .await?;
        }
    }

    for geofence in &previous_matches {
        if !current_ids.contains(&geofence.geofence_id) {
            insert_geofence_transition_event(
                pool,
                organization_id,
                device_id,
                geofence.geofence_id,
                "EXITED_ZONE",
                current_latitude,
                current_longitude,
                recorded_at,
            )
            .await?;
        }
    }

    Ok(())
}

pub async fn list_recent_geofence_transition_events(
    pool: &PgPool,
    device_id: Option<uuid::Uuid>,
) -> Result<Vec<GeofenceTransitionEventResponse>, sqlx::Error> {
    let events = sqlx::query_as!(
        GeofenceTransitionEventResponse,
        r#"
        SELECT
            gte.id,
            gte.organization_id,
            gte.device_id,

            gte.geofence_id,
            g.name AS geofence_name,
            g.geofence_type,

            gte.transition_type,

            gte.latitude,
            gte.longitude,

            gte.recorded_at,
            gte.detected_at,
            gte.created_at

        FROM geofence_transition_events gte
        INNER JOIN geofences g
            ON g.id = gte.geofence_id

        WHERE
            ($1::uuid IS NULL OR gte.device_id = $1)

        ORDER BY gte.detected_at DESC
        LIMIT 100
        "#,
        device_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(events)
}

pub async fn get_telemetry_history(
    pool: &PgPool,
    device_id: uuid::Uuid,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<TelemetryStreamResponse>, sqlx::Error> {
    let readings = sqlx::query_as!(
        TelemetryStreamResponse,
        r#"
        SELECT
            s.device_id,
            sr.recorded_at,
            sr.received_at,
            sr.value AS fuel_level_litres,
            sr.latitude,
            sr.longitude,
            sr.vibration_level,
            sr.motion_detected
        FROM sensor_readings sr
        JOIN sensors s ON s.id = sr.sensor_id
        WHERE
            s.device_id = $1
            AND sr.recorded_at >= $2
            AND sr.recorded_at <= $3
        ORDER BY sr.recorded_at ASC
        "#,
        device_id,
        start_time,
        end_time,
    )
    .fetch_all(pool)
    .await?;

    Ok(readings)
}

pub async fn get_geofence_activity_trends(
    db_pool: &PgPool,
    device_id: Option<Uuid>,
    days: i64,
) -> Result<GeofenceActivityTrendResponse, sqlx::Error> {
    let safe_days = if days < 1 {
        30
    } else if days > 90 {
        90
    } else {
        days
    };

    let trend_rows = sqlx::query!(
        r#"
        SELECT
            gte.detected_at::date AS "day!",
            COUNT(*) FILTER (
                WHERE gte.transition_type = 'ENTERED_ZONE'
            ) AS "entries!",
            COUNT(*) FILTER (
                WHERE gte.transition_type = 'EXITED_ZONE'
            ) AS "exits!"
        FROM geofence_transition_events gte
        WHERE
            gte.detected_at >= NOW() - ($2 * INTERVAL '1 day')
            AND (
                $1::uuid IS NULL
                OR gte.device_id = $1
            )
        GROUP BY gte.detected_at::date
        ORDER BY gte.detected_at::date ASC
        "#,
        device_id,
        safe_days as f64
    )
    .fetch_all(db_pool)
    .await?;

    let trend = trend_rows
        .into_iter()
        .map(|row| GeofenceActivityTrendPoint {
            day: row.day.to_string(),
            entries: row.entries,
            exits: row.exits,
        })
        .collect();

    Ok(GeofenceActivityTrendResponse {
        days: safe_days,
        trend,
    })
}

pub async fn get_device_health_trends(
    db_pool: &PgPool,
    days: i64,
) -> Result<DeviceHealthTrendResponse, sqlx::Error> {
    let safe_days = if days < 1 {
        30
    } else if days > 90 {
        90
    } else {
        days
    };

    let rows = sqlx::query!(
        r#"
        SELECT
            d.id AS "device_id!",
            d.device_code AS "device_code!",
            COUNT(*) FILTER (
                WHERE dhe.new_status = 'OFFLINE'
            ) AS "offline_events!",
            COUNT(*) FILTER (
                WHERE dhe.new_status = 'STALE'
            ) AS "stale_events!",
            COUNT(*) FILTER (
                WHERE dhe.new_status = 'ONLINE'
            ) AS "recovery_events!"
        FROM device_health_events dhe
        INNER JOIN devices d
            ON d.id = dhe.device_id
        WHERE
            dhe.created_at >= NOW() - ($1 * INTERVAL '1 day')
        GROUP BY
            d.id,
            d.device_code
        ORDER BY
            (
                COUNT(*) FILTER (
                    WHERE dhe.new_status IN ('OFFLINE', 'STALE')
                )
            ) DESC
        LIMIT 10
        "#,
        safe_days as f64
    )
    .fetch_all(db_pool)
    .await?;

    let devices = rows
        .into_iter()
        .map(|row| DeviceHealthTrendDevice {
            device_id: row.device_id,
            device_code: row.device_code,
            offline_events: row.offline_events,
            stale_events: row.stale_events,
            recovery_events: row.recovery_events,
            reliability_issue_count: row.offline_events + row.stale_events,
        })
        .collect();

    Ok(DeviceHealthTrendResponse {
        days: safe_days,
        devices,
    })
}

pub async fn get_geofence_utilization(
    db_pool: &PgPool,
    days: i64,
) -> Result<GeofenceUtilizationResponse, sqlx::Error> {
    let safe_days = if days < 1 {
        30
    } else if days > 90 {
        90
    } else {
        days
    };

    let rows = sqlx::query!(
        r#"
    SELECT
        g.name AS "geofence_name!",
        COUNT(*) FILTER (
            WHERE gte.transition_type = 'ENTERED_ZONE'
        ) AS "visits!"
    FROM geofence_transition_events gte
    INNER JOIN geofences g
        ON g.id = gte.geofence_id
    WHERE
        gte.detected_at >= NOW() - ($1 * INTERVAL '1 day')
    GROUP BY
        g.name
    ORDER BY
        COUNT(*) FILTER (
            WHERE gte.transition_type = 'ENTERED_ZONE'
        ) DESC,
        g.name ASC
    LIMIT 10
    "#,
        safe_days as f64
    )
    .fetch_all(db_pool)
    .await?;

    let zones = rows
        .into_iter()
        .map(|row| GeofenceUtilizationZone {
            geofence_name: row.geofence_name,
            visits: row.visits,
        })
        .collect();

    Ok(GeofenceUtilizationResponse {
        days: safe_days,
        zones,
    })
}
