use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::SensorCalibration;
use crate::repository;
use crate::services::calibration_cache::CalibrationCache;

/// Provides the application-wide entry point for retrieving active sensor
/// calibrations.
///
/// The loader hides where a calibration came from:
///
/// - in-memory cache
/// - PostgreSQL on a cache miss
///
/// Callers should use this loader instead of accessing the calibration cache
/// or calibration repository directly during telemetry processing.
#[derive(Debug, Clone)]
pub struct CalibrationLoader {
    db_pool: PgPool,
    cache: CalibrationCache,
}

impl CalibrationLoader {
    /// Creates a calibration loader using the default cache TTL.
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            cache: CalibrationCache::with_default_ttl(),
        }
    }

    /// Creates a calibration loader with a custom cache TTL.
    ///
    /// This is useful for tests or environments where calibrations must be
    /// refreshed more or less frequently.
    pub fn with_ttl(db_pool: PgPool, ttl: Duration) -> Self {
        Self {
            db_pool,
            cache: CalibrationCache::new(ttl),
        }
    }

    /// Retrieves the active calibration for one sensor and calibration type.
    ///
    /// Lookup order:
    ///
    /// 1. Return a valid cached result when one exists.
    /// 2. Query PostgreSQL on a cache miss.
    /// 3. Cache the database result.
    /// 4. Return the result to the caller.
    ///
    /// Missing calibrations are cached as well. This prevents repeated
    /// database queries for sensors that do not currently have an active
    /// calibration.
    pub async fn get_active(
        &self,
        sensor_id: Uuid,
        calibration_type: &str,
    ) -> Result<Option<Arc<SensorCalibration>>> {
        if let Some(cached_result) = self.cache.get(sensor_id, calibration_type).await {
            return Ok(cached_result);
        }

        let calibration =
            repository::get_active_sensor_calibration(&self.db_pool, sensor_id, calibration_type)
                .await?;

        match calibration {
            Some(calibration) => {
                self.cache.insert(calibration).await;

                // Read the value back from the cache so callers receive the
                // same shared Arc stored for future requests.
                let cached_result = self
                    .cache
                    .get(sensor_id, calibration_type)
                    .await
                    .expect("calibration was inserted into the cache");

                Ok(cached_result)
            }

            None => {
                self.cache.insert_missing(sensor_id, calibration_type).await;

                Ok(None)
            }
        }
    }

    /// Adds or replaces an active calibration in the cache.
    ///
    /// This will be useful after the calibration API successfully creates a
    /// new active calibration.
    pub async fn store(&self, calibration: SensorCalibration) {
        self.cache.insert(calibration).await;
    }

    /// Preloads active calibrations into memory.
    ///
    /// This will be called during application startup after all active
    /// calibration records have been loaded from PostgreSQL.
    pub async fn preload<I>(&self, calibrations: I)
    where
        I: IntoIterator<Item = SensorCalibration>,
    {
        self.cache.preload(calibrations).await;
    }

    /// Invalidates one calibration type for one sensor.
    ///
    /// The next call to `get_active` will reload the current active
    /// calibration from PostgreSQL.
    pub async fn invalidate(&self, sensor_id: Uuid, calibration_type: &str) {
        self.cache.invalidate(sensor_id, calibration_type).await;
    }

    /// Invalidates every cached calibration belonging to one sensor.
    pub async fn invalidate_sensor(&self, sensor_id: Uuid) {
        self.cache.invalidate_sensor(sensor_id).await;
    }

    /// Clears all cached calibration entries.
    pub async fn clear(&self) {
        self.cache.clear().await;
    }

    /// Returns the number of cache entries currently held.
    ///
    /// This includes cached missing-calibration results.
    pub async fn cached_entry_count(&self) -> usize {
        self.cache.len().await
    }

    pub async fn cache_is_empty(&self) -> bool {
        self.cache.is_empty().await
    }
}
