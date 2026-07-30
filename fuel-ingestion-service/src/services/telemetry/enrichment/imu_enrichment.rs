use std::sync::Arc;

use anyhow::Result;

use crate::{
    domain::{
        calibration::{CalibrationFactory, ImuCalibrationEngine},
        telemetry::models::TelemetryReading,
    },
    repository::RegisteredTelemetryContext,
    services::{calibration_loader::CalibrationLoader, calibration_type::CalibrationType},
};

/// Applies IMU-specific enrichments to canonical telemetry.
///
/// Current responsibility:
/// - Resolve the active IMU calibration.
/// - Decode the calibration into a typed domain model.
/// - Apply calibration to the IMU telemetry.
///
/// Future responsibilities may include:
/// - Gravity compensation.
/// - Mount orientation correction.
/// - Temperature compensation.
/// - Sensor-specific normalization.
#[derive(Clone)]
pub struct ImuEnrichment {
    calibration_loader: Arc<CalibrationLoader>,
}

impl ImuEnrichment {
    pub fn new(calibration_loader: Arc<CalibrationLoader>) -> Self {
        Self { calibration_loader }
    }

    pub async fn apply(
        &self,
        telemetry: &mut TelemetryReading,
        context: &RegisteredTelemetryContext,
    ) -> Result<()> {
        let Some(sensor_id) = context.vibration_sensor_id else {
            println!("IMU enrichment skipped: device has no registered vibration sensor");
            return Ok(());
        };

        let Some(imu) = telemetry.imu.as_mut() else {
            println!("IMU enrichment skipped: telemetry contains no IMU reading");
            return Ok(());
        };

        let Some(calibration) = self
            .calibration_loader
            .get_active(sensor_id, CalibrationType::Imu.as_str())
            .await?
        else {
            println!(
                "IMU enrichment skipped: no active IMU calibration for sensor {}",
                sensor_id
            );
            return Ok(());
        };

        let calibration = CalibrationFactory::imu(&calibration)?;

        let corrected = ImuCalibrationEngine::apply(imu, &calibration);

        println!(
            "IMU calibration applied for sensor {}: raw={:?}, corrected={:?}",
            sensor_id, imu, corrected
        );

        *imu = corrected;

        Ok(())
    }
}
