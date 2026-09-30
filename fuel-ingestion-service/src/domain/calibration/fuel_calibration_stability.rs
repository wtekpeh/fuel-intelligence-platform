/// Represents the current stability state of physical KUM observations
/// during a guided fuel-calibration measurement.
///
/// This state is derived from backend-observed physical telemetry.
/// The frontend may visualize the state, but it does not determine it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuelCalibrationStabilityState {
    /// There are not yet enough usable physical observations to begin
    /// evaluating measurement behaviour.
    WaitingForTelemetry,

    /// Enough telemetry is arriving to evaluate the measurement window,
    /// but there is not yet enough evidence to classify it as settled.
    Observing,

    /// The physical measurement is still changing beyond the currently
    /// acceptable stability conditions.
    Settling,

    /// The physical measurement has remained sufficiently stable across
    /// the required observation window.
    Stable,
}

/// One physical KUM observation supplied to the calibration stability
/// evaluator.
///
/// These values come from persisted physical fuel-sensor observations.
/// No tank-specific litres conversion has been applied.
#[derive(Debug, Clone, Copy)]
pub struct FuelCalibrationStabilityObservation {
    /// Authoritative KUM channel used for calibration-point capture.
    pub realtime_distance_cm: f64,

    /// Fast physical KUM channel retained as supporting stability evidence.
    pub raw_distance_cm: f64,

    /// Filtered KUM channel retained as supporting trend/stability evidence.
    pub smooth_distance_cm: f64,

    /// Observation time represented as milliseconds relative to the
    /// beginning of the evaluated window.
    ///
    /// Keeping the domain evaluator independent from database timestamp
    /// types makes the stability mathematics easier to test in isolation.
    pub elapsed_ms: u64,
}

/// Result produced by the calibration stability evaluator.
///
/// The result exposes both the state needed by the calibration workflow and
/// diagnostic measurements that can later drive the installer animation.
#[derive(Debug, Clone, Copy)]
pub struct FuelCalibrationStabilityResult {
    pub state: FuelCalibrationStabilityState,

    /// Number of usable physical observations considered.
    pub sample_count: usize,

    /// Elapsed time covered by the evaluated observation window.
    pub observation_duration_ms: u64,

    /// Difference between the largest and smallest authoritative real-time
    /// measurements in the evaluated window.
    pub realtime_range_cm: Option<f64>,

    /// Net rate of change of the authoritative real-time measurement across
    /// the evaluated window.
    ///
    /// This is intentionally represented in centimetres per second so it is
    /// meaningful independently of the firmware reporting cadence.
    pub realtime_slope_cm_per_second: Option<f64>,

    /// Authoritative real-time distance that may be used for automatic
    /// calibration capture once the state becomes Stable.
    pub capture_distance_cm: Option<f64>,
}

/// Configuration controlling how physical KUM observations are evaluated
/// for guided-calibration stability.
///
/// Keeping these values explicit allows the stability policy to be tuned from
/// physical hardware testing without changing the evaluator's mathematics.
#[derive(Debug, Clone, Copy)]
pub struct FuelCalibrationStabilityConfig {
    /// Minimum number of usable observations required before stability can
    /// be considered.
    pub minimum_sample_count: usize,

    /// Minimum amount of physical observation time required before the
    /// measurement can become stable.
    pub minimum_observation_duration_ms: u64,

    /// Maximum permitted range of the authoritative real-time KUM
    /// measurement across the evaluated stability window.
    pub maximum_realtime_range_cm: f64,

    /// Maximum permitted absolute rate of change of the authoritative
    /// real-time measurement.
    pub maximum_realtime_slope_cm_per_second: f64,
}

impl FuelCalibrationStabilityConfig {
    /// Creates an explicit stability configuration.
    pub const fn new(
        minimum_sample_count: usize,
        minimum_observation_duration_ms: u64,
        maximum_realtime_range_cm: f64,
        maximum_realtime_slope_cm_per_second: f64,
    ) -> Self {
        Self {
            minimum_sample_count,
            minimum_observation_duration_ms,
            maximum_realtime_range_cm,
            maximum_realtime_slope_cm_per_second,
        }
    }

    /// Current ORBI production policy for automatic guided fuel-calibration
    /// capture using the physical KUM ultrasonic sensor.
    ///
    /// These thresholds are intentionally expressed in measurement behaviour
    /// rather than tank volume:
    ///
    /// - sample count protects against isolated readings;
    /// - observation duration requires stability to persist through time;
    /// - real-time range limits short-window physical variation;
    /// - real-time slope prevents a changing fuel level from being captured
    ///   merely because individual readings happen to be close together.
    ///
    /// The policy can be refined as additional physical installations provide
    /// evidence across different vehicles and tank geometries without changing
    /// the stability evaluator itself.
    pub const fn production() -> Self {
        Self::new(
            3,      // at least three usable physical observations
            40_000, // stability must persist for at least 40 seconds
            0.35,   // maximum real-time spread: 0.35 cm
            0.01,   // maximum absolute trend: 0.01 cm/s
        )
    }
}

/// Evaluates a window of physical KUM observations and determines whether
/// the authoritative real-time fuel measurement is sufficiently stable for
/// automatic guided-calibration capture.
///
/// The evaluator contains no database, HTTP, firmware, or UI concerns.
/// It operates only on physical observation evidence supplied by the caller.
pub struct FuelCalibrationStabilityEvaluator;

impl FuelCalibrationStabilityEvaluator {
    pub fn evaluate(
        observations: &[FuelCalibrationStabilityObservation],
        config: FuelCalibrationStabilityConfig,
    ) -> FuelCalibrationStabilityResult {
        /*
         * No physical telemetry has arrived yet.
         */
        if observations.is_empty() {
            return FuelCalibrationStabilityResult {
                state: FuelCalibrationStabilityState::WaitingForTelemetry,
                sample_count: 0,
                observation_duration_ms: 0,
                realtime_range_cm: None,
                realtime_slope_cm_per_second: None,
                capture_distance_cm: None,
            };
        }

        /*
         * Only finite, physically meaningful measurements are usable by
         * the stability evaluator.
         *
         * The repository already protects persisted distances from negative
         * values, but the domain layer still protects itself so it remains
         * correct when tested or called independently.
         */
        let usable_observations: Vec<&FuelCalibrationStabilityObservation> = observations
            .iter()
            .filter(|observation| {
                observation.realtime_distance_cm.is_finite()
                    && observation.realtime_distance_cm >= 0.0
                    && observation.raw_distance_cm.is_finite()
                    && observation.raw_distance_cm >= 0.0
                    && observation.smooth_distance_cm.is_finite()
                    && observation.smooth_distance_cm >= 0.0
            })
            .collect();

        if usable_observations.is_empty() {
            return FuelCalibrationStabilityResult {
                state: FuelCalibrationStabilityState::WaitingForTelemetry,
                sample_count: 0,
                observation_duration_ms: 0,
                realtime_range_cm: None,
                realtime_slope_cm_per_second: None,
                capture_distance_cm: None,
            };
        }

        let sample_count = usable_observations.len();

        /*
         * elapsed_ms is relative to the beginning of the observation window.
         * Using the minimum and maximum values means the evaluator does not
         * depend on the caller supplying observations in a particular order.
         */
        let minimum_elapsed_ms = usable_observations
            .iter()
            .map(|observation| observation.elapsed_ms)
            .min()
            .unwrap_or(0);

        let maximum_elapsed_ms = usable_observations
            .iter()
            .map(|observation| observation.elapsed_ms)
            .max()
            .unwrap_or(minimum_elapsed_ms);

        let observation_duration_ms = maximum_elapsed_ms.saturating_sub(minimum_elapsed_ms);

        /*
         * Until both the required sample count and observation duration have
         * been reached, there is not enough evidence to make a stability
         * decision.
         */
        if sample_count < config.minimum_sample_count
            || observation_duration_ms < config.minimum_observation_duration_ms
        {
            return FuelCalibrationStabilityResult {
                state: FuelCalibrationStabilityState::Observing,
                sample_count,
                observation_duration_ms,
                realtime_range_cm: None,
                realtime_slope_cm_per_second: None,
                capture_distance_cm: None,
            };
        }

        let minimum_realtime_cm = usable_observations
            .iter()
            .map(|observation| observation.realtime_distance_cm)
            .fold(f64::INFINITY, f64::min);

        let maximum_realtime_cm = usable_observations
            .iter()
            .map(|observation| observation.realtime_distance_cm)
            .fold(f64::NEG_INFINITY, f64::max);

        let realtime_range_cm = maximum_realtime_cm - minimum_realtime_cm;

        /*
         * Determine the oldest and newest usable observations by elapsed
         * measurement time.
         */
        let oldest_observation = usable_observations
            .iter()
            .min_by_key(|observation| observation.elapsed_ms)
            .expect("usable observations are known to be non-empty");

        let newest_observation = usable_observations
            .iter()
            .max_by_key(|observation| observation.elapsed_ms)
            .expect("usable observations are known to be non-empty");

        let duration_seconds = observation_duration_ms as f64 / 1_000.0;

        let realtime_slope_cm_per_second = if duration_seconds > 0.0 {
            (newest_observation.realtime_distance_cm - oldest_observation.realtime_distance_cm)
                / duration_seconds
        } else {
            0.0
        };

        let range_is_stable = realtime_range_cm <= config.maximum_realtime_range_cm;

        let slope_is_stable =
            realtime_slope_cm_per_second.abs() <= config.maximum_realtime_slope_cm_per_second;

        if !range_is_stable || !slope_is_stable {
            return FuelCalibrationStabilityResult {
                state: FuelCalibrationStabilityState::Settling,
                sample_count,
                observation_duration_ms,
                realtime_range_cm: Some(realtime_range_cm),
                realtime_slope_cm_per_second: Some(realtime_slope_cm_per_second),
                capture_distance_cm: None,
            };
        }

        /*
         * Stability has been established.
         *
         * The newest authoritative real-time KUM measurement becomes the
         * candidate calibration distance. Raw and smooth measurements remain
         * supporting evidence only.
         */
        FuelCalibrationStabilityResult {
            state: FuelCalibrationStabilityState::Stable,
            sample_count,
            observation_duration_ms,
            realtime_range_cm: Some(realtime_range_cm),
            realtime_slope_cm_per_second: Some(realtime_slope_cm_per_second),
            capture_distance_cm: Some(newest_observation.realtime_distance_cm),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FuelCalibrationStabilityConfig {
        /*
         * These values are test fixtures only.
         *
         * They make the expected state transitions easy to reason about.
         * They are NOT ORBI production stability thresholds.
         */
        FuelCalibrationStabilityConfig::new(
            5,      // minimum samples
            20_000, // minimum observation duration: 20 seconds
            0.30,   // maximum real-time range: 0.30 cm
            0.01,   // maximum absolute slope: 0.01 cm/s
        )
    }

    fn observation(
        elapsed_ms: u64,
        realtime_distance_cm: f64,
    ) -> FuelCalibrationStabilityObservation {
        FuelCalibrationStabilityObservation {
            realtime_distance_cm,
            raw_distance_cm: realtime_distance_cm,
            smooth_distance_cm: realtime_distance_cm,
            elapsed_ms,
        }
    }

    #[test]
    fn no_observations_waits_for_telemetry() {
        let result = FuelCalibrationStabilityEvaluator::evaluate(&[], test_config());

        assert_eq!(
            result.state,
            FuelCalibrationStabilityState::WaitingForTelemetry
        );

        assert_eq!(result.sample_count, 0);
        assert_eq!(result.observation_duration_ms, 0);
        assert_eq!(result.realtime_range_cm, None);
        assert_eq!(result.realtime_slope_cm_per_second, None);
        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn insufficient_observation_window_remains_observing() {
        let observations = [
            observation(0, 20.10),
            observation(5_000, 20.11),
            observation(10_000, 20.09),
            observation(15_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Observing);

        assert_eq!(result.sample_count, 4);
        assert_eq!(result.observation_duration_ms, 15_000);
        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn changing_measurement_is_settling() {
        let observations = [
            observation(0, 18.00),
            observation(5_000, 18.40),
            observation(10_000, 18.80),
            observation(15_000, 19.20),
            observation(20_000, 19.60),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Settling);

        assert_eq!(result.sample_count, 5);
        assert_eq!(result.observation_duration_ms, 20_000);
        assert_eq!(result.capture_distance_cm, None);

        assert!(
            result.realtime_range_cm.expect("range should be available")
                > test_config().maximum_realtime_range_cm
        );

        assert!(
            result
                .realtime_slope_cm_per_second
                .expect("slope should be available")
                .abs()
                > test_config().maximum_realtime_slope_cm_per_second
        );
    }

    #[test]
    fn sufficiently_stable_measurement_becomes_stable() {
        let observations = [
            observation(0, 20.10),
            observation(5_000, 20.12),
            observation(10_000, 20.09),
            observation(15_000, 20.11),
            observation(20_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Stable);

        assert_eq!(result.sample_count, 5);
        assert_eq!(result.observation_duration_ms, 20_000);

        assert!(result.realtime_range_cm.is_some());
        assert!(result.realtime_slope_cm_per_second.is_some());

        assert_eq!(result.capture_distance_cm, Some(20.10));
    }

    #[test]
    fn enough_samples_without_enough_time_remains_observing() {
        let observations = [
            observation(0, 20.10),
            observation(1_000, 20.11),
            observation(2_000, 20.10),
            observation(3_000, 20.09),
            observation(4_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Observing);

        assert_eq!(result.sample_count, 5);
        assert_eq!(result.observation_duration_ms, 4_000);
        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn enough_time_without_enough_samples_remains_observing() {
        let observations = [
            observation(0, 20.10),
            observation(10_000, 20.11),
            observation(20_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Observing);

        assert_eq!(result.sample_count, 3);
        assert_eq!(result.observation_duration_ms, 20_000);
        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn observation_order_does_not_change_stability_result() {
        let observations = [
            observation(20_000, 20.10),
            observation(5_000, 20.12),
            observation(15_000, 20.11),
            observation(0, 20.10),
            observation(10_000, 20.09),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Stable);

        assert_eq!(result.observation_duration_ms, 20_000);

        /*
         * Capture must use the newest observation by elapsed time,
         * not whichever element happens to appear last in the slice.
         */
        assert_eq!(result.capture_distance_cm, Some(20.10));
    }

    #[test]
    fn invalid_physical_observations_are_excluded() {
        let observations = [
            observation(0, 20.10),
            observation(5_000, f64::NAN),
            observation(10_000, 20.11),
            observation(15_000, 20.09),
            observation(20_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        /*
         * One invalid observation leaves only four usable samples.
         * Therefore the evaluator must not declare stability.
         */
        assert_eq!(result.state, FuelCalibrationStabilityState::Observing);

        assert_eq!(result.sample_count, 4);
        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn excessive_range_prevents_stable_capture() {
        let observations = [
            observation(0, 20.00),
            observation(5_000, 20.35),
            observation(10_000, 20.05),
            observation(15_000, 20.30),
            observation(20_000, 20.10),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, test_config());

        assert_eq!(result.state, FuelCalibrationStabilityState::Settling);

        assert!(
            result.realtime_range_cm.expect("range should be available")
                > test_config().maximum_realtime_range_cm
        );

        assert_eq!(result.capture_distance_cm, None);
    }

    #[test]
    fn excessive_trend_prevents_stable_capture() {
        /*
         * Give this test a permissive range threshold so that slope alone
         * determines the result.
         */
        let config = FuelCalibrationStabilityConfig::new(5, 20_000, 10.0, 0.01);

        let observations = [
            observation(0, 20.00),
            observation(5_000, 20.10),
            observation(10_000, 20.20),
            observation(15_000, 20.30),
            observation(20_000, 20.40),
        ];

        let result = FuelCalibrationStabilityEvaluator::evaluate(&observations, config);

        assert_eq!(result.state, FuelCalibrationStabilityState::Settling);

        assert!(
            result
                .realtime_slope_cm_per_second
                .expect("slope should be available")
                .abs()
                > config.maximum_realtime_slope_cm_per_second
        );

        assert_eq!(result.capture_distance_cm, None);
    }
}
