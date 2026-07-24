use super::{
    imu_interpreter::{ImuInterpretation, interpret_imu},
    models::TelemetryReading,
    motion_buffer::{MotionEvidence, MotionSample},
    motion_tracker::MotionTracker,
};

/// Processes canonical ORBI telemetry and coordinates the
/// operational interpretation of measurements.
///
/// This pipeline operates only on the canonical TelemetryReading model.
/// It does not depend on HTTP handlers, database repositories,
/// legacy ingestion models, or firmware-specific payload structures.
#[derive(Debug, Default)]
pub struct TelemetryPipeline {
    motion_tracker: MotionTracker,
}

/// Result produced when one canonical telemetry reading
/// containing IMU data is processed.
#[derive(Debug, Clone)]
pub struct ProcessedTelemetry {
    /// Interpretation derived from the current raw IMU sample.
    pub imu_interpretation: ImuInterpretation,

    /// Rolling motion evidence after the current sample was added.
    pub motion_evidence: MotionEvidence,
}

impl TelemetryPipeline {
    /// Creates an empty telemetry pipeline.
    pub fn new() -> Self {
        Self {
            motion_tracker: MotionTracker::new(),
        }
    }

    /// Processes one canonical telemetry reading.
    ///
    /// When IMU telemetry is present:
    ///
    /// 1. The raw IMU measurement is interpreted.
    /// 2. The interpretation is added to the device's rolling buffer.
    /// 3. The latest aggregated motion evidence is returned.
    ///
    /// When IMU telemetry is absent, no motion evidence is produced and
    /// the existing device motion history remains unchanged.
    pub fn process(&mut self, telemetry: &TelemetryReading) -> Option<ProcessedTelemetry> {
        let imu = telemetry.imu.as_ref()?;

        let imu_interpretation = interpret_imu(imu);

        let sample = MotionSample {
            recorded_at: telemetry.recorded_at,
            interpretation: imu_interpretation.clone(),
        };

        let motion_evidence = self
            .motion_tracker
            .update(telemetry.device_id.clone(), sample);

        Some(ProcessedTelemetry {
            imu_interpretation,
            motion_evidence,
        })
    }

    /// Returns the latest motion evidence for a device without
    /// adding another telemetry sample.
    pub fn motion_evidence(&self, device_id: &str) -> Option<MotionEvidence> {
        self.motion_tracker.evidence(device_id)
    }

    /// Returns the number of motion samples currently retained
    /// for a particular device.
    pub fn motion_sample_count(&self, device_id: &str) -> usize {
        self.motion_tracker.sample_count(device_id)
    }

    /// Returns true when a device has accumulated a complete
    /// rolling motion window.
    pub fn has_full_motion_window(&self, device_id: &str) -> bool {
        self.motion_tracker.has_full_window(device_id)
    }

    /// Returns true when motion history exists for a device.
    pub fn contains_device(&self, device_id: &str) -> bool {
        self.motion_tracker.contains_device(device_id)
    }

    /// Returns the number of devices currently held by the
    /// in-memory motion tracker.
    pub fn tracked_device_count(&self) -> usize {
        self.motion_tracker.device_count()
    }

    /// Removes the retained motion history for one device.
    pub fn remove_device(&mut self, device_id: &str) -> bool {
        self.motion_tracker.remove_device(device_id)
    }

    /// Removes all retained motion histories.
    pub fn clear(&mut self) {
        self.motion_tracker.clear();
    }

    /// Returns true when the pipeline contains no device motion history.
    pub fn is_empty(&self) -> bool {
        self.motion_tracker.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};

    use crate::domain::telemetry::models::{
        DiagnosticTelemetry, ImuTelemetry, PositionTelemetry, TelemetryReading,
    };

    fn create_telemetry(
        device_id: &str,
        second: u32,
        imu: Option<ImuTelemetry>,
    ) -> TelemetryReading {
        TelemetryReading {
            device_id: device_id.to_string(),

            recorded_at: Utc
                .with_ymd_and_hms(2026, 1, 1, 0, 0, second)
                .single()
                .expect("test timestamp should be valid"),

            position: Some(PositionTelemetry {
                latitude: 5.6037,
                longitude: -0.1870,
                altitude: None,
                heading: Some(0.0),
                speed_kmh: Some(0.0),
                satellite_count: None,
                hdop: None,
            }),

            fuel: None,

            imu,

            power: None,

            diagnostics: Some(DiagnosticTelemetry {
                firmware_version: None,
                signal_strength: None,
                queued_records: None,
                modem_temperature: None,
            }),
        }
    }

    fn stationary_imu() -> ImuTelemetry {
        ImuTelemetry {
            accel_x: 0.0,
            accel_y: 0.0,
            accel_z: 1.0,

            gyro_x: 0.0,
            gyro_y: 0.0,
            gyro_z: 0.0,

            temperature: Some(27.0),
        }
    }

    fn moving_imu() -> ImuTelemetry {
        ImuTelemetry {
            accel_x: 0.8,
            accel_y: 0.8,
            accel_z: 0.8,

            gyro_x: 8.0,
            gyro_y: 4.0,
            gyro_z: 3.0,

            temperature: Some(27.0),
        }
    }

    #[test]
    fn new_pipeline_contains_no_motion_history() {
        let pipeline = TelemetryPipeline::new();

        assert!(pipeline.is_empty());
        assert_eq!(pipeline.tracked_device_count(), 0);
        assert!(!pipeline.contains_device("ORBI-GPS-001"));
        assert_eq!(pipeline.motion_sample_count("ORBI-GPS-001"), 0);
        assert!(pipeline.motion_evidence("ORBI-GPS-001").is_none());
    }

    #[test]
    fn telemetry_without_imu_produces_no_motion_evidence() {
        let mut pipeline = TelemetryPipeline::new();

        let telemetry = create_telemetry("ORBI-GPS-001", 1, None);

        let evidence = pipeline.process(&telemetry);

        assert!(evidence.is_none());
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.motion_sample_count("ORBI-GPS-001"), 0);
    }

    #[test]
    fn stationary_imu_produces_stationary_motion_evidence() {
        let mut pipeline = TelemetryPipeline::new();

        let telemetry = create_telemetry("ORBI-GPS-001", 1, Some(stationary_imu()));

        let processed = pipeline
            .process(&telemetry)
            .expect("IMU telemetry should produce processed telemetry");

        let evidence = processed.motion_evidence;

        assert_eq!(evidence.sample_count, 1);
        assert_eq!(evidence.average_vibration_score, 0.0);
        assert_eq!(evidence.motion_ratio, 0.0);
        assert_eq!(evidence.average_confidence, 0.0);
        assert!(!evidence.sustained_motion);

        assert!(pipeline.contains_device("ORBI-GPS-001"));
        assert_eq!(pipeline.tracked_device_count(), 1);
    }

    #[test]
    fn moving_imu_produces_positive_motion_evidence() {
        let mut pipeline = TelemetryPipeline::new();

        let telemetry = create_telemetry("ORBI-GPS-001", 1, Some(moving_imu()));

        let processed = pipeline
            .process(&telemetry)
            .expect("IMU telemetry should produce processed telemetry");

        let evidence = processed.motion_evidence;

        assert_eq!(evidence.sample_count, 1);
        assert!(evidence.average_vibration_score >= 2.0);
        assert_eq!(evidence.motion_ratio, 1.0);
        assert!(evidence.average_confidence > 0.0);
        assert!(evidence.sustained_motion);
    }

    #[test]
    fn repeated_readings_build_a_complete_motion_window() {
        let mut pipeline = TelemetryPipeline::new();

        for second in 1..=5 {
            let telemetry = create_telemetry("ORBI-GPS-001", second, Some(moving_imu()));

            pipeline.process(&telemetry);
        }

        assert_eq!(pipeline.motion_sample_count("ORBI-GPS-001"), 5);
        assert!(pipeline.has_full_motion_window("ORBI-GPS-001"));

        let evidence = pipeline
            .motion_evidence("ORBI-GPS-001")
            .expect("device should have accumulated motion evidence");

        assert_eq!(evidence.sample_count, 5);
        assert_eq!(evidence.motion_ratio, 1.0);
        assert!(evidence.sustained_motion);
    }

    #[test]
    fn different_devices_keep_independent_motion_histories() {
        let mut pipeline = TelemetryPipeline::new();

        let moving = create_telemetry("ORBI-GPS-001", 1, Some(moving_imu()));

        let stationary = create_telemetry("ORBI-GPS-002", 1, Some(stationary_imu()));

        pipeline.process(&moving);
        pipeline.process(&stationary);

        assert_eq!(pipeline.tracked_device_count(), 2);
        assert_eq!(pipeline.motion_sample_count("ORBI-GPS-001"), 1);
        assert_eq!(pipeline.motion_sample_count("ORBI-GPS-002"), 1);

        let moving_evidence = pipeline
            .motion_evidence("ORBI-GPS-001")
            .expect("moving device should have evidence");

        let stationary_evidence = pipeline
            .motion_evidence("ORBI-GPS-002")
            .expect("stationary device should have evidence");

        assert!(moving_evidence.sustained_motion);
        assert!(!stationary_evidence.sustained_motion);
    }

    #[test]
    fn removing_device_clears_only_its_motion_history() {
        let mut pipeline = TelemetryPipeline::new();

        let first = create_telemetry("ORBI-GPS-001", 1, Some(moving_imu()));

        let second = create_telemetry("ORBI-GPS-002", 1, Some(stationary_imu()));

        pipeline.process(&first);
        pipeline.process(&second);

        let removed = pipeline.remove_device("ORBI-GPS-001");

        assert!(removed);
        assert!(!pipeline.contains_device("ORBI-GPS-001"));
        assert!(pipeline.contains_device("ORBI-GPS-002"));
        assert_eq!(pipeline.tracked_device_count(), 1);
    }

    #[test]
    fn clear_removes_all_motion_histories() {
        let mut pipeline = TelemetryPipeline::new();

        let first = create_telemetry("ORBI-GPS-001", 1, Some(moving_imu()));

        let second = create_telemetry("ORBI-GPS-002", 1, Some(stationary_imu()));

        pipeline.process(&first);
        pipeline.process(&second);

        pipeline.clear();

        assert!(pipeline.is_empty());
        assert_eq!(pipeline.tracked_device_count(), 0);
        assert!(pipeline.motion_evidence("ORBI-GPS-001").is_none());
        assert!(pipeline.motion_evidence("ORBI-GPS-002").is_none());
    }
}
