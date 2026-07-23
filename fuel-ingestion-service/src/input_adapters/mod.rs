// Converts vendor/protocol-specific payloads into ORBI's canonical telemetry models.

pub mod registry;
pub mod traits;

// Existing adapter used by the legacy fuel ingestion endpoint.
pub mod legacy_fuel_reading;
