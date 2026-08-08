use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::{
    domain::{
        calibration::{CalibrationFactory, FuelCalibrationEngine},
        telemetry::models::CalibratedFuelTelemetry,
    },
    services::{calibration_loader::CalibrationLoader, calibration_type::CalibrationType},
};

/// Coordinates runtime fuel calibration.
///
/// This service is responsible for:
///
/// - resolving the active fuel calibration for an installed sensor;
/// - converting the stored JSON calibration into a typed domain model;
/// - applying lookup-table interpolation;
/// - returning a calibrated fuel quantity when the measured level is
///   inside the currently verified calibration range.
///
/// It does not:
///
/// - persist telemetry;
/// - detect theft, refill, or leakage;
/// - manage guided calibration sessions;
/// - modify calibration profiles.
#[derive(Clone)]
pub struct FuelCalibrationService {
    calibration_loader: Arc<CalibrationLoader>,
}

impl FuelCalibrationService {
    pub fn new(calibration_loader: Arc<CalibrationLoader>) -> Self {
        Self { calibration_loader }
    }

    /// Converts a KUM liquid-level measurement into litres and percentage.
    ///
    /// Returns `Ok(None)` when:
    ///
    /// - the sensor has no active fuel calibration; or
    /// - the measured level is below the lowest verified calibration point.
    pub async fn calibrate(
        &self,
        sensor_id: Uuid,
        measured_level_cm: f64,
    ) -> Result<Option<CalibratedFuelTelemetry>> {
        let Some(calibration_record) = self
            .calibration_loader
            .get_active(sensor_id, CalibrationType::Fuel.as_str())
            .await?
        else {
            return Ok(None);
        };

        let calibration = CalibrationFactory::fuel(&calibration_record)?;

        FuelCalibrationEngine::apply(measured_level_cm, &calibration)
    }
}
