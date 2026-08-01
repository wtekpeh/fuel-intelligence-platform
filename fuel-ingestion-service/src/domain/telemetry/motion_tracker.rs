use std::collections::HashMap;

use super::motion_buffer::{MotionBuffer, MotionEvidence, MotionSample};

/// Manages independent rolling motion histories for multiple devices.
///
/// Every device receives its own MotionBuffer so telemetry arriving
/// from different devices cannot affect another device's evidence.
#[derive(Debug, Default)]
pub struct MotionTracker {
    buffers: HashMap<String, MotionBuffer>,
}

impl MotionTracker {
    /// Creates an empty fleet-wide motion tracker.
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    /// Adds a new interpreted motion sample to the appropriate
    /// device-specific rolling buffer.
    ///
    /// A new buffer is automatically created when the device has
    /// not previously submitted an IMU sample.
    ///
    /// The latest aggregated motion evidence is returned after the
    /// sample has been added.
    pub fn update(
        &mut self,
        device_code: impl Into<String>,
        sample: MotionSample,
    ) -> MotionEvidence {
        let device_code = device_code.into();

        let buffer = self
            .buffers
            .entry(device_code)
            .or_insert_with(MotionBuffer::new);

        buffer.push(sample);

        buffer
            .evidence()
            .expect("a buffer containing a newly added sample must produce motion evidence")
    }

    /// Returns the current motion evidence for a device without
    /// modifying its rolling history.
    ///
    /// None is returned when the device has no motion history.
    pub fn evidence(&self, device_code: &str) -> Option<MotionEvidence> {
        self.buffers
            .get(device_code)
            .and_then(MotionBuffer::evidence)
    }

    /// Returns the number of motion samples currently retained
    /// for a particular device.
    ///
    /// A device without a buffer has zero retained samples.
    pub fn sample_count(&self, device_code: &str) -> usize {
        self.buffers
            .get(device_code)
            .map(MotionBuffer::len)
            .unwrap_or(0)
    }

    /// Returns true when the device has accumulated a complete
    /// rolling motion window.
    pub fn has_full_window(&self, device_code: &str) -> bool {
        self.buffers
            .get(device_code)
            .map(MotionBuffer::is_full)
            .unwrap_or(false)
    }

    /// Returns true when motion history currently exists for
    /// the supplied device code.
    pub fn contains_device(&self, device_code: &str) -> bool {
        self.buffers.contains_key(device_code)
    }

    /// Returns the total number of devices that currently have
    /// an in-memory motion history.
    pub fn device_count(&self) -> usize {
        self.buffers.len()
    }

    /// Removes the rolling motion history belonging to one device.
    ///
    /// Returns true when a device buffer existed and was removed.
    pub fn remove_device(&mut self, device_code: &str) -> bool {
        self.buffers.remove(device_code).is_some()
    }

    /// Removes motion history for every device.
    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    /// Returns true when no device motion histories are retained.
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};

    use crate::domain::telemetry::{
        imu_interpreter::ImuInterpretation, physical_motion_metrics::PhysicalMotionMetrics,
    };

    fn create_interpretation(
        vibration_score: f64,
        motion_detected: bool,
        movement_confidence: f64,
    ) -> ImuInterpretation {
        ImuInterpretation {
            physical: PhysicalMotionMetrics::new(1.0, 0.0, 0.0),
            dynamic_acceleration_g: 0.0,
            vibration_score,
            motion_detected,
            movement_confidence,
        }
    }

    fn create_sample(
        second: u32,
        vibration_score: f64,
        motion_detected: bool,
        movement_confidence: f64,
    ) -> MotionSample {
        MotionSample {
            recorded_at: Utc
                .with_ymd_and_hms(2026, 1, 1, 0, 0, second)
                .single()
                .expect("test timestamp should be valid"),

            interpretation: create_interpretation(
                vibration_score,
                motion_detected,
                movement_confidence,
            ),
        }
    }

    fn assert_close(actual: f64, expected: f64) {
        let difference = (actual - expected).abs();

        assert!(
            difference < 0.000_001,
            "expected {expected}, received {actual}"
        );
    }

    #[test]
    fn new_tracker_contains_no_devices() {
        let tracker = MotionTracker::new();

        assert!(tracker.is_empty());
        assert_eq!(tracker.device_count(), 0);
        assert!(!tracker.contains_device("ORBI-GPS-001"));
        assert_eq!(tracker.sample_count("ORBI-GPS-001"), 0);
        assert!(!tracker.has_full_window("ORBI-GPS-001"));
        assert!(tracker.evidence("ORBI-GPS-001").is_none());
    }

    #[test]
    fn update_creates_a_buffer_for_a_new_device() {
        let mut tracker = MotionTracker::new();

        let evidence = tracker.update("ORBI-GPS-001", create_sample(1, 4.0, true, 0.8));

        assert!(tracker.contains_device("ORBI-GPS-001"));
        assert_eq!(tracker.device_count(), 1);
        assert_eq!(tracker.sample_count("ORBI-GPS-001"), 1);

        assert_eq!(evidence.sample_count, 1);
        assert_close(evidence.average_vibration_score, 4.0);
        assert_close(evidence.motion_ratio, 1.0);
        assert_close(evidence.average_confidence, 0.8);
        assert!(evidence.sustained_motion);
    }

    #[test]
    fn devices_have_independent_motion_buffers() {
        let mut tracker = MotionTracker::new();

        tracker.update("ORBI-GPS-001", create_sample(1, 4.0, true, 0.8));

        tracker.update("ORBI-GPS-001", create_sample(2, 6.0, true, 1.0));

        tracker.update("ORBI-GPS-002", create_sample(1, 0.0, false, 0.0));

        assert_eq!(tracker.device_count(), 2);
        assert_eq!(tracker.sample_count("ORBI-GPS-001"), 2);
        assert_eq!(tracker.sample_count("ORBI-GPS-002"), 1);

        let device_one_evidence = tracker
            .evidence("ORBI-GPS-001")
            .expect("first device should have evidence");

        let device_two_evidence = tracker
            .evidence("ORBI-GPS-002")
            .expect("second device should have evidence");

        assert_close(device_one_evidence.average_vibration_score, 5.0);

        assert_close(device_two_evidence.average_vibration_score, 0.0);

        assert!(device_one_evidence.sustained_motion);
        assert!(!device_two_evidence.sustained_motion);
    }

    #[test]
    fn one_device_reaching_full_window_does_not_affect_another() {
        let mut tracker = MotionTracker::new();

        for second in 1..=5 {
            tracker.update("ORBI-GPS-001", create_sample(second, 4.0, true, 0.8));
        }

        tracker.update("ORBI-GPS-002", create_sample(1, 0.0, false, 0.0));

        assert!(tracker.has_full_window("ORBI-GPS-001"));
        assert!(!tracker.has_full_window("ORBI-GPS-002"));

        assert_eq!(tracker.sample_count("ORBI-GPS-001"), 5);
        assert_eq!(tracker.sample_count("ORBI-GPS-002"), 1);
    }

    #[test]
    fn device_buffer_retains_only_the_latest_five_samples() {
        let mut tracker = MotionTracker::new();

        for second in 1..=6 {
            tracker.update(
                "ORBI-GPS-001",
                create_sample(second, second as f64, true, 1.0),
            );
        }

        let evidence = tracker
            .evidence("ORBI-GPS-001")
            .expect("device should have motion evidence");

        assert_eq!(tracker.sample_count("ORBI-GPS-001"), 5);
        assert!(tracker.has_full_window("ORBI-GPS-001"));

        // The retained vibration scores are:
        //
        // 2, 3, 4, 5, 6
        //
        // Their average is 4.0.
        assert_close(evidence.average_vibration_score, 4.0);
    }

    #[test]
    fn remove_device_deletes_only_that_devices_history() {
        let mut tracker = MotionTracker::new();

        tracker.update("ORBI-GPS-001", create_sample(1, 4.0, true, 0.8));

        tracker.update("ORBI-GPS-002", create_sample(1, 0.0, false, 0.0));

        let removed = tracker.remove_device("ORBI-GPS-001");

        assert!(removed);
        assert!(!tracker.contains_device("ORBI-GPS-001"));
        assert!(tracker.contains_device("ORBI-GPS-002"));
        assert_eq!(tracker.device_count(), 1);
    }

    #[test]
    fn removing_an_unknown_device_returns_false() {
        let mut tracker = MotionTracker::new();

        let removed = tracker.remove_device("UNKNOWN-DEVICE");

        assert!(!removed);
        assert!(tracker.is_empty());
    }

    #[test]
    fn clear_removes_all_device_histories() {
        let mut tracker = MotionTracker::new();

        tracker.update("ORBI-GPS-001", create_sample(1, 4.0, true, 0.8));

        tracker.update("ORBI-GPS-002", create_sample(1, 0.0, false, 0.0));

        tracker.clear();

        assert!(tracker.is_empty());
        assert_eq!(tracker.device_count(), 0);
        assert!(tracker.evidence("ORBI-GPS-001").is_none());
        assert!(tracker.evidence("ORBI-GPS-002").is_none());
    }
}
