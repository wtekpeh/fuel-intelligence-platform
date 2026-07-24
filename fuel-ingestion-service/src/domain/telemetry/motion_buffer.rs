use std::collections::VecDeque;

use chrono::{DateTime, Utc};

use super::imu_interpreter::ImuInterpretation;

/// Number of interpreted IMU samples retained for motion evidence.
///
/// Five samples provide a small rolling history that smooths
/// transient spikes without introducing noticeable latency.
pub const MOTION_WINDOW_SIZE: usize = 5;

/// One interpreted motion sample.
///
/// This intentionally stores backend-derived motion evidence
/// rather than raw IMU measurements.
#[derive(Debug, Clone)]
pub struct MotionSample {
    pub recorded_at: DateTime<Utc>,
    pub interpretation: ImuInterpretation,
}

/// Aggregated evidence produced from a rolling history
/// of interpreted IMU samples.
#[derive(Debug, Clone)]
pub struct MotionEvidence {
    /// Mean vibration score across the current window.
    pub average_vibration_score: f64,

    /// Fraction of samples indicating motion.
    ///
    /// Example:
    /// 4 motion samples out of 5 = 0.8
    pub motion_ratio: f64,

    /// Mean confidence across the window.
    pub average_confidence: f64,

    /// True when motion is sustained across most
    /// of the rolling window.
    pub sustained_motion: bool,

    /// Number of samples currently contributing.
    pub sample_count: usize,
}

/// Rolling buffer of interpreted motion samples.
#[derive(Debug, Default)]
pub struct MotionBuffer {
    samples: VecDeque<MotionSample>,
}

impl MotionBuffer {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(MOTION_WINDOW_SIZE),
        }
    }

    /// Add one interpreted IMU sample.
    pub fn push(&mut self, sample: MotionSample) {
        if self.samples.len() == MOTION_WINDOW_SIZE {
            self.samples.pop_front();
        }

        self.samples.push_back(sample);
    }

    /// Returns true once the buffer has reached
    /// the configured rolling window size.
    pub fn is_full(&self) -> bool {
        self.samples.len() == MOTION_WINDOW_SIZE
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Aggregate the current motion history.
    pub fn evidence(&self) -> Option<MotionEvidence> {
        if self.samples.is_empty() {
            return None;
        }

        let sample_count = self.samples.len();

        let average_vibration_score = self
            .samples
            .iter()
            .map(|sample| sample.interpretation.vibration_score)
            .sum::<f64>()
            / sample_count as f64;

        let average_confidence = self
            .samples
            .iter()
            .map(|sample| sample.interpretation.movement_confidence)
            .sum::<f64>()
            / sample_count as f64;

        let motion_samples = self
            .samples
            .iter()
            .filter(|sample| sample.interpretation.motion_detected)
            .count();

        let motion_ratio = motion_samples as f64 / sample_count as f64;

        Some(MotionEvidence {
            average_vibration_score,
            motion_ratio,
            average_confidence,
            sustained_motion: motion_ratio >= 0.6,
            sample_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_interpretation(
        vibration_score: f64,
        motion_detected: bool,
        movement_confidence: f64,
    ) -> ImuInterpretation {
        ImuInterpretation {
            acceleration_magnitude_g: 1.0,
            dynamic_acceleration_g: 0.0,
            rotation_magnitude_dps: 0.0,
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
    fn empty_buffer_returns_no_motion_evidence() {
        let buffer = MotionBuffer::new();

        assert_eq!(buffer.len(), 0);
        assert!(!buffer.is_full());
        assert!(buffer.evidence().is_none());
    }

    #[test]
    fn single_sample_produces_matching_motion_evidence() {
        let mut buffer = MotionBuffer::new();

        buffer.push(create_sample(1, 4.0, true, 0.8));

        let evidence = buffer
            .evidence()
            .expect("one sample should produce motion evidence");

        assert_eq!(evidence.sample_count, 1);
        assert_close(evidence.average_vibration_score, 4.0);
        assert_close(evidence.motion_ratio, 1.0);
        assert_close(evidence.average_confidence, 0.8);
        assert!(evidence.sustained_motion);
    }

    #[test]
    fn buffer_never_exceeds_the_motion_window_size() {
        let mut buffer = MotionBuffer::new();

        for second in 1..=6 {
            buffer.push(create_sample(second, second as f64, true, 1.0));
        }

        assert_eq!(buffer.len(), MOTION_WINDOW_SIZE);
        assert!(buffer.is_full());
    }

    #[test]
    fn oldest_sample_is_removed_when_window_is_full() {
        let mut buffer = MotionBuffer::new();

        for second in 1..=6 {
            buffer.push(create_sample(second, second as f64, true, 1.0));
        }

        let evidence = buffer
            .evidence()
            .expect("full buffer should produce motion evidence");

        // After adding samples 1 through 6 to a five-sample buffer,
        // the retained vibration scores must be:
        //
        // 2, 3, 4, 5, 6
        //
        // Their average is 4.0.
        assert_eq!(evidence.sample_count, 5);
        assert_close(evidence.average_vibration_score, 4.0);
    }

    #[test]
    fn motion_ratio_is_calculated_from_detected_samples() {
        let mut buffer = MotionBuffer::new();

        buffer.push(create_sample(1, 3.0, true, 0.7));
        buffer.push(create_sample(2, 3.0, true, 0.7));
        buffer.push(create_sample(3, 0.0, false, 0.0));
        buffer.push(create_sample(4, 3.0, true, 0.7));
        buffer.push(create_sample(5, 0.0, false, 0.0));

        let evidence = buffer
            .evidence()
            .expect("buffer should produce motion evidence");

        // Three out of five samples indicate motion.
        assert_close(evidence.motion_ratio, 0.6);
        assert!(evidence.sustained_motion);
    }

    #[test]
    fn motion_below_sixty_percent_is_not_sustained() {
        let mut buffer = MotionBuffer::new();

        buffer.push(create_sample(1, 3.0, true, 0.7));
        buffer.push(create_sample(2, 3.0, true, 0.7));
        buffer.push(create_sample(3, 0.0, false, 0.0));
        buffer.push(create_sample(4, 0.0, false, 0.0));
        buffer.push(create_sample(5, 0.0, false, 0.0));

        let evidence = buffer
            .evidence()
            .expect("buffer should produce motion evidence");

        assert_close(evidence.motion_ratio, 0.4);
        assert!(!evidence.sustained_motion);
    }

    #[test]
    fn averages_are_calculated_across_the_current_window() {
        let mut buffer = MotionBuffer::new();

        buffer.push(create_sample(1, 1.0, false, 0.2));
        buffer.push(create_sample(2, 2.0, true, 0.4));
        buffer.push(create_sample(3, 3.0, true, 0.6));
        buffer.push(create_sample(4, 4.0, true, 0.8));
        buffer.push(create_sample(5, 5.0, true, 1.0));

        let evidence = buffer
            .evidence()
            .expect("buffer should produce motion evidence");

        assert_eq!(evidence.sample_count, 5);
        assert_close(evidence.average_vibration_score, 3.0);
        assert_close(evidence.average_confidence, 0.6);
        assert_close(evidence.motion_ratio, 0.8);
        assert!(evidence.sustained_motion);
    }
}
