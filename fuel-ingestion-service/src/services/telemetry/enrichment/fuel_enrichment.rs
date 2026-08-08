use anyhow::Result;

use crate::{
    domain::telemetry::models::TelemetryReading, repository::RegisteredTelemetryContext,
    services::fuel_calibration_service::FuelCalibrationService,
};
/// Applies fuel-specific enrichments to canonical telemetry.
///
/// Current responsibility:
/// - Load the active fuel sensor calibration.
///
/// Future responsibilities may include:
/// - Applying calibration coefficients.
/// - Converting sensor distance to fuel height.
/// - Applying tank geometry.
/// - Applying temperature compensation.
/// - Producing calibrated litres and percentage.
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

        let calibrated = self
            .calibration_service
            .calibrate(sensor_id, fuel.raw.smooth_distance_cm)
            .await?;

        match calibrated {
            Some(calibrated_fuel) => {
                fuel.calibrated = Some(calibrated_fuel);
            }

            None => {
                /*
                 * No verified calibration currently exists for the measured
                 * liquid level.
                 *
                 * Preserve the raw KUM measurement while leaving the
                 * calibrated quantity unavailable.
                 */
                fuel.calibrated = None;

                println!(
                    "Fuel calibration unavailable below verified range for sensor {}: level={} cm",
                    sensor_id, fuel.raw.smooth_distance_cm,
                );
            }
        }
        Ok(())
    }
}
