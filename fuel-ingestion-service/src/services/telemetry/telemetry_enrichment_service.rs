use std::sync::Arc;

use anyhow::Result;

use crate::services::fuel_calibration_service::FuelCalibrationService;
use crate::{
    domain::telemetry::models::TelemetryReading,
    repository::RegisteredTelemetryContext,
    services::{
        calibration_loader::CalibrationLoader,
        telemetry::enrichment::{fuel_enrichment::FuelEnrichment, imu_enrichment::ImuEnrichment},
    },
};

#[derive(Clone)]
pub struct TelemetryEnrichmentService {
    fuel: FuelEnrichment,
    imu: ImuEnrichment,
}

impl TelemetryEnrichmentService {
    pub fn new(calibration_loader: Arc<CalibrationLoader>) -> Self {
        /*
         * Construct shared domain services.
         */
        let fuel_calibration_service = FuelCalibrationService::new(calibration_loader.clone());

        /*
         * Construct telemetry enrichments.
         */
        let fuel = FuelEnrichment::new(fuel_calibration_service);

        let imu = ImuEnrichment::new(calibration_loader);

        Self { fuel, imu }
    }
    /// Applies every telemetry enrichment before telemetry
    /// enters the operational intelligence pipeline.
    pub async fn enrich(
        &self,
        telemetry: &TelemetryReading,
        context: &RegisteredTelemetryContext,
    ) -> Result<TelemetryReading> {
        let mut enriched = telemetry.clone();

        self.fuel.apply(&mut enriched, context).await?;

        self.imu.apply(&mut enriched, context).await?;

        Ok(enriched)
    }
}
