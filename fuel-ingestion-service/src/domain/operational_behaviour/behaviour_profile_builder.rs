use crate::domain::operational_behaviour::{BehaviourProfileStatistics, BehaviourSample};

/// Minimum number of behaviour samples required
/// before a statistically meaningful profile can
/// be produced.
pub const MINIMUM_SAMPLE_COUNT: usize = 30;

/// Stateless builder responsible for transforming
/// behaviour samples into statistical summaries.
///
/// This type contains no persistence logic and
/// performs no database interaction.
#[derive(Debug, Default)]
pub struct BehaviourProfileBuilder;

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviourProfileBuildError {
    InsufficientSamples { required: usize, actual: usize },
}

impl BehaviourProfileBuilder {
    pub fn build(
        samples: &[BehaviourSample],
    ) -> Result<BehaviourProfileStatistics, BehaviourProfileBuildError> {
        if samples.len() < MINIMUM_SAMPLE_COUNT {
            return Err(BehaviourProfileBuildError::InsufficientSamples {
                required: MINIMUM_SAMPLE_COUNT,
                actual: samples.len(),
            });
        }

        let mut mean = 0.0;
        let mut m2 = 0.0;

        let mut min_vibration = f64::INFINITY;
        let mut max_vibration = f64::NEG_INFINITY;

        let mut gravity_mean = 0.0;
        let mut gravity_m2 = 0.0;

        let mut gravity_min = f64::INFINITY;
        let mut gravity_max = f64::NEG_INFINITY;

        let mut rotation_mean = 0.0;
        let mut rotation_m2 = 0.0;

        let mut rotation_min = f64::INFINITY;
        let mut rotation_max = f64::NEG_INFINITY;

        let mut motion_sum = 0.0;
        let mut motion_min = f64::INFINITY;
        let mut motion_max = f64::NEG_INFINITY;

        let mut confidence_sum = 0.0;

        let mut sustained_count = 0usize;

        let mut gps_sum = 0.0;
        let mut gps_count = 0usize;

        for (index, sample) in samples.iter().enumerate() {
            let vibration = sample.motion_evidence.average_vibration_score;

            let gravity = sample.motion_evidence.average_gravity_deviation_g;

            let rotation = sample.motion_evidence.average_rotation_magnitude_dps;

            let n = index as f64 + 1.0;

            let delta = vibration - mean;
            mean += delta / n;
            let delta2 = vibration - mean;
            m2 += delta * delta2;

            let gravity_delta = gravity - gravity_mean;
            gravity_mean += gravity_delta / n;
            let gravity_delta2 = gravity - gravity_mean;
            gravity_m2 += gravity_delta * gravity_delta2;

            let rotation_delta = rotation - rotation_mean;
            rotation_mean += rotation_delta / n;
            let rotation_delta2 = rotation - rotation_mean;
            rotation_m2 += rotation_delta * rotation_delta2;

            min_vibration = min_vibration.min(vibration);
            max_vibration = max_vibration.max(vibration);

            gravity_min = gravity_min.min(gravity);
            gravity_max = gravity_max.max(gravity);

            rotation_min = rotation_min.min(rotation);
            rotation_max = rotation_max.max(rotation);

            let motion = sample.motion_evidence.motion_ratio;

            motion_sum += motion;
            motion_min = motion_min.min(motion);
            motion_max = motion_max.max(motion);

            confidence_sum += sample.motion_evidence.average_confidence;

            if sample.motion_evidence.sustained_motion {
                sustained_count += 1;
            }

            if let Some(speed) = sample.gps_speed_kmh {
                gps_sum += speed;
                gps_count += 1;
            }
        }

        let sample_count = samples.len();

        let variance = m2 / sample_count as f64;

        let gravity_variance = gravity_m2 / sample_count as f64;

        let rotation_variance = rotation_m2 / sample_count as f64;

        Ok(BehaviourProfileStatistics {
            sample_count,

            average_vibration_score: mean,

            minimum_vibration_score: min_vibration,

            maximum_vibration_score: max_vibration,

            vibration_variance: variance,

            vibration_standard_deviation: variance.sqrt(),

            average_gravity_deviation_g: gravity_mean,

            minimum_gravity_deviation_g: gravity_min,

            maximum_gravity_deviation_g: gravity_max,

            gravity_deviation_variance: gravity_variance,

            gravity_deviation_standard_deviation: gravity_variance.sqrt(),

            average_rotation_magnitude_dps: rotation_mean,

            minimum_rotation_magnitude_dps: rotation_min,

            maximum_rotation_magnitude_dps: rotation_max,

            rotation_magnitude_variance: rotation_variance,

            rotation_magnitude_standard_deviation: rotation_variance.sqrt(),

            average_motion_ratio: motion_sum / sample_count as f64,

            minimum_motion_ratio: motion_min,

            maximum_motion_ratio: motion_max,

            average_confidence: confidence_sum / sample_count as f64,

            sustained_motion_ratio: sustained_count as f64 / sample_count as f64,

            average_gps_speed_kmh: if gps_count == 0 {
                None
            } else {
                Some(gps_sum / gps_count as f64)
            },
        })
    }
}
