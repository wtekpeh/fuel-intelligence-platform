use std::sync::Arc;

use anyhow::Result;

use crate::{
    domain::telemetry::models::TelemetryReading,
    repository::RegisteredTelemetryContext,
    services::{calibration_loader::CalibrationLoader, calibration_type::CalibrationType},
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
    calibration_loader: Arc<CalibrationLoader>,
}

impl FuelEnrichment {
    pub fn new(calibration_loader: Arc<CalibrationLoader>) -> Self {
        Self { calibration_loader }
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

        let Some(_fuel) = telemetry.fuel.as_mut() else {
            return Ok(());
        };

        let Some(_calibration) = self
            .calibration_loader
            .get_active(sensor_id, CalibrationType::Fuel.as_str())
            .await?
        else {
            return Ok(());
        };

        // The active calibration has now been resolved.
        //
        // Calibration mathematics will be implemented after we define
        // and validate the supported calibration_values JSON formats.
        //
        // `_fuel` and `_calibration` are intentionally retained here
        // because they will become the inputs to that transformation.

        Ok(())
    }
}
