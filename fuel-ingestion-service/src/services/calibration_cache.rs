use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::SensorCalibration;

/// Uniquely identifies one calibration category for one provisioned sensor.
///
/// A sensor may have multiple active calibration categories, for example:
///
/// - OFFSET
/// - LINEAR
/// - TANK_GEOMETRY
/// - IMU_BIAS
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CalibrationCacheKey {
    pub sensor_id: Uuid,
    pub calibration_type: String,
}

impl CalibrationCacheKey {
    pub fn new(sensor_id: Uuid, calibration_type: impl Into<String>) -> Self {
        Self {
            sensor_id,
            calibration_type: normalize_calibration_type(calibration_type.into()),
        }
    }
}

/// Represents one value held by the cache.
///
/// `calibration` is optional intentionally:
///
/// - `Some(...)` means an active calibration was found.
/// - `None` means the database was checked but no active calibration existed.
///
/// Caching `None` prevents repeated database queries for sensors that do not
/// currently have a calibration profile.
#[derive(Debug)]
struct CachedCalibration {
    calibration: Option<Arc<SensorCalibration>>,
    loaded_at: Instant,
}

impl CachedCalibration {
    fn new(calibration: Option<Arc<SensorCalibration>>) -> Self {
        Self {
            calibration,
            loaded_at: Instant::now(),
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.loaded_at.elapsed() >= ttl
    }
}

/// Shared in-memory cache for active sensor calibrations.
///
/// Cloning this type does not duplicate the calibration data. Every clone
/// refers to the same underlying cache through `Arc<RwLock<...>>`.
#[derive(Debug, Clone)]
pub struct CalibrationCache {
    entries: Arc<RwLock<HashMap<CalibrationCacheKey, CachedCalibration>>>,
    ttl: Duration,
}

impl CalibrationCache {
    /// Creates an empty cache with the supplied entry lifetime.
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Creates an empty cache using the recommended initial five-minute TTL.
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(5 * 60))
    }

    /// Returns a cached calibration when the key exists and has not expired.
    ///
    /// The outer `Option` describes whether a valid cache entry exists.
    /// The inner `Option` describes whether an active calibration exists.
    ///
    /// Results:
    ///
    /// - `Some(Some(calibration))` — active calibration cached
    /// - `Some(None)` — absence of calibration cached
    /// - `None` — cache miss or expired entry
    pub async fn get(
        &self,
        sensor_id: Uuid,
        calibration_type: &str,
    ) -> Option<Option<Arc<SensorCalibration>>> {
        let key = CalibrationCacheKey::new(sensor_id, calibration_type);

        {
            let entries = self.entries.read().await;

            if let Some(entry) = entries.get(&key) {
                if !entry.is_expired(self.ttl) {
                    return Some(entry.calibration.clone());
                }
            }
        }

        // The entry either does not exist or has expired.
        // Remove expired data so that the next loader call can refresh it.
        self.entries.write().await.remove(&key);

        None
    }

    /// Stores an active calibration.
    pub async fn insert(&self, calibration: SensorCalibration) {
        let key =
            CalibrationCacheKey::new(calibration.sensor_id, calibration.calibration_type.clone());

        let cached = CachedCalibration::new(Some(Arc::new(calibration)));

        self.entries.write().await.insert(key, cached);
    }

    /// Stores the fact that no active calibration currently exists.
    pub async fn insert_missing(&self, sensor_id: Uuid, calibration_type: &str) {
        let key = CalibrationCacheKey::new(sensor_id, calibration_type);
        let cached = CachedCalibration::new(None);

        self.entries.write().await.insert(key, cached);
    }

    /// Loads or replaces several active calibrations at once.
    ///
    /// This will be used during application startup after active calibration
    /// records have been loaded from PostgreSQL.
    pub async fn preload<I>(&self, calibrations: I)
    where
        I: IntoIterator<Item = SensorCalibration>,
    {
        let mut entries = self.entries.write().await;

        for calibration in calibrations {
            let key = CalibrationCacheKey::new(
                calibration.sensor_id,
                calibration.calibration_type.clone(),
            );

            entries.insert(key, CachedCalibration::new(Some(Arc::new(calibration))));
        }
    }

    /// Invalidates one calibration category for a sensor.
    ///
    /// This should be called after a new calibration is created or activated.
    pub async fn invalidate(&self, sensor_id: Uuid, calibration_type: &str) {
        let key = CalibrationCacheKey::new(sensor_id, calibration_type);

        self.entries.write().await.remove(&key);
    }

    /// Invalidates every cached calibration belonging to one sensor.
    pub async fn invalidate_sensor(&self, sensor_id: Uuid) {
        self.entries
            .write()
            .await
            .retain(|key, _| key.sensor_id != sensor_id);
    }

    /// Removes every calibration from the cache.
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Returns the number of entries currently held by the cache.
    ///
    /// This includes cached missing-calibration entries.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }
}

impl Default for CalibrationCache {
    fn default() -> Self {
        Self::with_default_ttl()
    }
}

fn normalize_calibration_type(calibration_type: impl AsRef<str>) -> String {
    calibration_type.as_ref().trim().to_uppercase()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use tokio::time::sleep;

    use super::*;

    fn sample_calibration(sensor_id: Uuid, calibration_type: &str) -> SensorCalibration {
        let now = Utc::now();

        SensorCalibration {
            id: Uuid::new_v4(),
            sensor_id,
            calibration_type: calibration_type.to_string(),
            calibration_values: json!({
                "offset": 1.5
            }),
            is_active: true,
            calibrated_at: now,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn stores_and_returns_active_calibration() {
        let cache = CalibrationCache::with_default_ttl();
        let sensor_id = Uuid::new_v4();

        cache.insert(sample_calibration(sensor_id, "OFFSET")).await;

        let result = cache.get(sensor_id, "offset").await;

        let calibration = result
            .expect("cache entry should exist")
            .expect("active calibration should exist");

        assert_eq!(calibration.sensor_id, sensor_id);
        assert_eq!(calibration.calibration_type, "OFFSET");
    }

    #[tokio::test]
    async fn stores_missing_calibration_result() {
        let cache = CalibrationCache::with_default_ttl();
        let sensor_id = Uuid::new_v4();

        cache.insert_missing(sensor_id, "LINEAR").await;

        let result = cache.get(sensor_id, "linear").await;

        assert!(matches!(result, Some(None)));
    }

    #[tokio::test]
    async fn returns_none_for_cache_miss() {
        let cache = CalibrationCache::with_default_ttl();

        let result = cache.get(Uuid::new_v4(), "OFFSET").await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn invalidates_one_calibration() {
        let cache = CalibrationCache::with_default_ttl();
        let sensor_id = Uuid::new_v4();

        cache.insert(sample_calibration(sensor_id, "OFFSET")).await;

        cache.invalidate(sensor_id, "OFFSET").await;

        assert!(cache.get(sensor_id, "OFFSET").await.is_none());
    }

    #[tokio::test]
    async fn invalidates_all_calibrations_for_sensor() {
        let cache = CalibrationCache::with_default_ttl();
        let sensor_id = Uuid::new_v4();

        cache.insert(sample_calibration(sensor_id, "OFFSET")).await;

        cache.insert(sample_calibration(sensor_id, "LINEAR")).await;

        cache.invalidate_sensor(sensor_id).await;

        assert!(cache.get(sensor_id, "OFFSET").await.is_none());
        assert!(cache.get(sensor_id, "LINEAR").await.is_none());
    }

    #[tokio::test]
    async fn expired_entry_becomes_cache_miss() {
        let cache = CalibrationCache::new(Duration::from_millis(10));
        let sensor_id = Uuid::new_v4();

        cache.insert(sample_calibration(sensor_id, "OFFSET")).await;

        sleep(Duration::from_millis(20)).await;

        assert!(cache.get(sensor_id, "OFFSET").await.is_none());
    }
}
