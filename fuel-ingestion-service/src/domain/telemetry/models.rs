use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Root telemetry packet exchanged between ORBI Firmware
/// and the ORBI Sensor Intelligence Platform.
///
/// A telemetry packet contains measurements only.
/// Operational intelligence is derived by backend services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryReading {
    pub device_id: String,
    pub recorded_at: DateTime<Utc>,

    pub position: Option<PositionTelemetry>,
    pub fuel: Option<FuelTelemetry>,
    pub imu: Option<ImuTelemetry>,
    pub power: Option<PowerTelemetry>,
    pub diagnostics: Option<DiagnosticTelemetry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionTelemetry {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub heading: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub satellite_count: Option<u8>,
    pub hdop: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelTelemetry {
    /// Raw physical measurement received from the fuel sensor.
    ///
    /// This remains available even before tank calibration has
    /// been applied.
    pub raw: RawFuelTelemetry,

    /// Calibrated tank values produced by the backend.
    ///
    /// This is `None` until a valid tank calibration profile has
    /// converted the raw sensor measurement into litres and
    /// percentage.
    pub calibrated: Option<CalibratedFuelTelemetry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFuelTelemetry {
    /// Smoothed ultrasonic distance between the sensor and the
    /// detected liquid surface.
    pub smooth_distance_cm: f64,

    /// Current real-time ultrasonic distance.
    pub realtime_distance_cm: f64,

    /// Unfiltered native ultrasonic distance.
    pub raw_distance_cm: f64,

    /// Temperature reported by the KUM sensor.
    pub temperature_c: f64,

    /// Sensor-specific status byte returned by the KUM protocol.
    pub status_byte_1: u8,

    /// Sensor-specific status byte returned by the KUM protocol.
    pub status_byte_2: u8,

    /// Raw-data validity value returned by the KUM sensor.
    pub raw_data_validity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibratedFuelTelemetry {
    /// Fuel volume produced by backend tank calibration.
    pub litres: f64,

    /// Tank fill percentage produced by backend calibration.
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImuTelemetry {
    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,

    pub gyro_x: f64,
    pub gyro_y: f64,
    pub gyro_z: f64,

    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerTelemetry {
    pub supply_voltage: Option<f64>,
    pub battery_voltage: Option<f64>,
    pub ignition_on: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTelemetry {
    pub firmware_version: Option<String>,
    pub signal_strength: Option<i32>,
    pub queued_records: Option<u32>,
    pub modem_temperature: Option<f64>,
}
