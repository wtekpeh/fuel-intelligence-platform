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
    pub litres: f64,
    pub percentage: f64,

    /// Native value reported by the sensor.
    /// This may represent different things depending
    /// on the sensor technology (RS485, ultrasonic,
    /// capacitive, CAN, etc.).
    pub sensor_value: Option<f64>,

    pub temperature: Option<f64>,
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
