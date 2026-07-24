// src/input_adapters/traits.rs

use anyhow::Result;

use crate::domain::telemetry::models::TelemetryReading;

/// Converts vendor- or protocol-specific payloads into
/// ORBI's canonical telemetry model.
///
/// Implementations should perform parsing, validation,
/// and normalization only.
///
/// Business logic (fuel theft, movement detection,
/// analytics, alerting, persistence) belongs elsewhere.
pub trait TelemetryAdapter {
    /// Human-readable adapter name.
    fn name(&self) -> &'static str;

    /// Vendor or protocol identifier.
    fn vendor(&self) -> &'static str;

    /// Convert an external payload into ORBI's canonical
    /// telemetry representation.
    fn adapt(&self, payload: &[u8]) -> Result<TelemetryReading>;
}
