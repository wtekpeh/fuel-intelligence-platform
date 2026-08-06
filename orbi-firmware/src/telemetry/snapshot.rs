use crate::drivers::{gnss::GpsInfo, kum::KumMeasurement, vibration::ImuData};

/// Represents one complete set of physical measurements collected
/// during a single telemetry sampling cycle.
///
/// The snapshot deliberately contains only physical sensor
/// measurements.
///
/// It contains no derived values, operational intelligence,
/// alerts, or analytics.
///
/// Sensors that are not installed on a particular hardware profile
/// are represented as `None`.
pub struct SensorSnapshot<'a> {
    /// GNSS measurement.
    pub gps: &'a GpsInfo,

    /// IMU measurement.
    pub imu: &'a ImuData,

    /// Fuel measurement (optional because not every ORBI device
    /// includes a KUM sensor).
    pub fuel: Option<&'a KumMeasurement>,
}
