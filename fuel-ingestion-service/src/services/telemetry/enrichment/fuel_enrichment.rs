use anyhow::Result;

use crate::{
    domain::telemetry::models::TelemetryReading, repository::RegisteredTelemetryContext,
    services::fuel_calibration_service::FuelCalibrationService,
};
/// Applies fuel-specific enrichments to canonical telemetry.
///
/// Current responsibility:
///
/// - Load the active fuel calibration for the installed FUEL sensor.
/// - Convert the KUM real-time level measurement into calibrated litres
///   and percentage.
///
/// Runtime measurement policy:
///
/// - `realtime_distance_cm` is the canonical input for live fuel
///   calibration because physical testing showed that it reacts quickly
///   to actual liquid-level changes.
/// - `smooth_distance_cm` is retained for diagnostics, calibration
///   stability checks, and trend/reference analysis, but is deliberately
///   not used for live fuel quantity because it may lag by several minutes.
/// - `raw_distance_cm` is retained as the fastest low-level sensor
///   observation and for diagnostics/validation, but is not currently
///   used directly as the runtime calibration input.
///
/// Future responsibilities may include:
///
/// - Sensor-validity gating.
/// - Short-window confirmation for theft/refill detection.
/// - Temperature compensation.
/// - More advanced tank-specific filtering.
#[derive(Clone)]
pub struct FuelEnrichment {
    calibration_service: FuelCalibrationService,
}

impl FuelEnrichment {
    pub fn new(calibration_service: FuelCalibrationService) -> Self {
        Self {
            calibration_service,
        }
    }

    /// Applies fuel enrichment directly to the supplied canonical telemetry.
    ///
    /// The method safely performs no work when:
    /// - The registered device has no fuel sensor.
    /// - The telemetry packet contains no fuel measurement.
    /// - The fuel sensor has no active calibration.
    pub async fn apply(
        &self,
        telemetry: &mut TelemetryReading,
        context: &RegisteredTelemetryContext,
    ) -> Result<()> {
        let Some(sensor_id) = context.fuel_sensor_id else {
            return Ok(());
        };

        let Some(fuel) = telemetry.fuel.as_mut() else {
            return Ok(());
        };

        /*
         * Runtime fuel calibration intentionally uses the KUM real-time
         * measurement.
         *
         * Physical hardware testing showed that:
         *
         * - raw reacts essentially immediately;
         * - real-time closely tracks raw;
         * - smooth may remain at the previous liquid level for several minutes.
         *
         * Using smooth here would therefore delay live fuel quantity changes and
         * could delay theft/refill detection.
         */
        let calibrated = self
            .calibration_service
            .calibrate(sensor_id, fuel.raw.realtime_distance_cm)
            .await?;

        match calibrated {
            Some(calibrated_fuel) => {
                fuel.calibrated = Some(calibrated_fuel);
            }

            None => {
                /*
                 * A calibrated quantity is currently unavailable.
                 *
                 * This may mean:
                 *
                 * - the sensor has no active runtime fuel calibration; or
                 * - the measured level is below the lowest verified point.
                 *
                 * Preserve the raw KUM measurements and leave calibrated fuel
                 * quantity unavailable.
                 */
                fuel.calibrated = None;

                println!(
                    "Fuel calibration unavailable for sensor {}: realtime_level={} cm",
                    sensor_id, fuel.raw.realtime_distance_cm,
                );
            }
        }
        Ok(())
    }
}
