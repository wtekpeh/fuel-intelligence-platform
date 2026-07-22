use anyhow::Result;
use sqlx::PgPool;

use crate::repository::{self, NewSensorReading};

/// Persists a normalized sensor reading.
///
/// Sensor-specific services are responsible for converting their
/// measurements into `NewSensorReading`. This function provides the
/// shared persistence path used by all telemetry types.
pub async fn persist_sensor_reading(db_pool: &PgPool, new_reading: NewSensorReading) -> Result<()> {
    repository::insert_sensor_reading(db_pool, new_reading).await
}
