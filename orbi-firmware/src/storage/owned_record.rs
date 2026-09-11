use heapless::String;

use crate::{
    scheduler::reporting::MotionState, storage::record::GnssDiagnosticRecord,
    telemetry::record::TelemetryRecord,
};

pub struct OwnedTelemetryRecord {
    pub device_id: String<64>,
    pub timestamp: String<32>,

    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub heading: f64,

    pub fuel_distance_smooth_cm: f32,
    pub fuel_distance_realtime_cm: f32,
    pub fuel_distance_raw_cm: f32,
    pub fuel_sensor_temperature_c: f32,
    pub fuel_sensor_status_1: u8,
    pub fuel_sensor_status_2: u8,
    pub fuel_raw_data_validity: u8,

    pub accel_x_g: f32,
    pub accel_y_g: f32,
    pub accel_z_g: f32,

    pub gyro_x_dps: f32,
    pub gyro_y_dps: f32,
    pub gyro_z_dps: f32,

    pub imu_temperature_c: f32,

    pub simulation_mode: String<32>,
}

impl OwnedTelemetryRecord {
    pub fn as_borrowed(&self) -> TelemetryRecord<'_> {
        TelemetryRecord {
            device_id: self.device_id.as_str(),
            timestamp: self.timestamp.as_str(),

            latitude: self.latitude,
            longitude: self.longitude,
            speed: self.speed,
            heading: self.heading,

            fuel_distance_smooth_cm: self.fuel_distance_smooth_cm,
            fuel_distance_realtime_cm: self.fuel_distance_realtime_cm,
            fuel_distance_raw_cm: self.fuel_distance_raw_cm,
            fuel_sensor_temperature_c: self.fuel_sensor_temperature_c,
            fuel_sensor_status_1: self.fuel_sensor_status_1,
            fuel_sensor_status_2: self.fuel_sensor_status_2,
            fuel_raw_data_validity: self.fuel_raw_data_validity,

            accel_x_g: self.accel_x_g,
            accel_y_g: self.accel_y_g,
            accel_z_g: self.accel_z_g,

            gyro_x_dps: self.gyro_x_dps,
            gyro_y_dps: self.gyro_y_dps,
            gyro_z_dps: self.gyro_z_dps,

            imu_temperature_c: self.imu_temperature_c,

            simulation_mode: self.simulation_mode.as_str(),
        }
    }

    pub fn from_borrowed(record: &TelemetryRecord<'_>) -> Option<Self> {
        let mut device_id = String::<64>::new();

        if device_id.push_str(record.device_id).is_err() {
            return None;
        }

        let mut timestamp = String::<32>::new();

        if timestamp.push_str(record.timestamp).is_err() {
            return None;
        }

        let mut simulation_mode = String::<32>::new();

        if simulation_mode.push_str(record.simulation_mode).is_err() {
            return None;
        }

        Some(Self {
            device_id,
            timestamp,

            latitude: record.latitude,
            longitude: record.longitude,
            speed: record.speed,
            heading: record.heading,

            fuel_distance_smooth_cm: record.fuel_distance_smooth_cm,

            fuel_distance_realtime_cm: record.fuel_distance_realtime_cm,

            fuel_distance_raw_cm: record.fuel_distance_raw_cm,

            fuel_sensor_temperature_c: record.fuel_sensor_temperature_c,

            fuel_sensor_status_1: record.fuel_sensor_status_1,

            fuel_sensor_status_2: record.fuel_sensor_status_2,

            fuel_raw_data_validity: record.fuel_raw_data_validity,

            accel_x_g: record.accel_x_g,
            accel_y_g: record.accel_y_g,
            accel_z_g: record.accel_z_g,

            gyro_x_dps: record.gyro_x_dps,
            gyro_y_dps: record.gyro_y_dps,
            gyro_z_dps: record.gyro_z_dps,

            imu_temperature_c: record.imu_temperature_c,

            simulation_mode,
        })
    }
}

pub struct OwnedGnssDiagnosticRecord {
    pub timestamp: String<32>,

    pub latitude: f64,
    pub longitude: f64,

    pub speed_knots: f64,
    pub speed_kmh: f64,

    pub heading_degrees: f64,

    pub motion_state: MotionState,
    pub reporting_interval_seconds: u32,
}

impl OwnedGnssDiagnosticRecord {
    pub fn from_borrowed(record: &GnssDiagnosticRecord<'_>) -> Option<Self> {
        let mut timestamp = String::<32>::new();

        if timestamp.push_str(record.timestamp).is_err() {
            return None;
        }

        Some(Self {
            timestamp,

            latitude: record.latitude,
            longitude: record.longitude,

            speed_knots: record.speed_knots,
            speed_kmh: record.speed_kmh,

            heading_degrees: record.heading_degrees,

            motion_state: record.motion_state,

            reporting_interval_seconds: record.reporting_interval_seconds,
        })
    }

    pub fn as_borrowed(&self) -> GnssDiagnosticRecord<'_> {
        GnssDiagnosticRecord {
            timestamp: self.timestamp.as_str(),

            latitude: self.latitude,
            longitude: self.longitude,

            speed_knots: self.speed_knots,
            speed_kmh: self.speed_kmh,

            heading_degrees: self.heading_degrees,

            motion_state: self.motion_state,
            reporting_interval_seconds: self.reporting_interval_seconds,
        }
    }
}

pub struct OwnedAckRecord {
    pub device_id: String<64>,
    pub timestamp: String<32>,
}

impl OwnedAckRecord {
    pub fn device_id(&self) -> &str {
        self.device_id.as_str()
    }

    pub fn timestamp(&self) -> &str {
        self.timestamp.as_str()
    }

    pub fn new(device_id: &str, timestamp: &str) -> Option<Self> {
        let mut owned_device_id = String::<64>::new();

        if owned_device_id.push_str(device_id).is_err() {
            return None;
        }

        let mut owned_timestamp = String::<32>::new();

        if owned_timestamp.push_str(timestamp).is_err() {
            return None;
        }

        Some(Self {
            device_id: owned_device_id,
            timestamp: owned_timestamp,
        })
    }
}
