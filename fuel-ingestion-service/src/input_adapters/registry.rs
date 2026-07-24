use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};

use super::traits::TelemetryAdapter;

/// Stores the telemetry adapters available to the ingestion service.
///
/// Adapter keys should be stable identifiers such as:
///
/// - `legacy-fuel-json`
/// - `kum-modbus`
/// - `nmea-gps`
/// - `j1939-can`
pub struct AdapterRegistry {
    adapters: HashMap<String, Arc<dyn TelemetryAdapter + Send + Sync>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Registers an adapter under a stable identifier.
    ///
    /// Registering the same identifier twice replaces the existing adapter.
    pub fn register<A>(&mut self, adapter_id: impl Into<String>, adapter: A)
    where
        A: TelemetryAdapter + Send + Sync + 'static,
    {
        self.adapters.insert(adapter_id.into(), Arc::new(adapter));
    }

    /// Returns an adapter by its stable identifier.
    pub fn get(&self, adapter_id: &str) -> Result<Arc<dyn TelemetryAdapter + Send + Sync>> {
        self.adapters
            .get(adapter_id)
            .cloned()
            .ok_or_else(|| anyhow!("telemetry adapter not registered: {adapter_id}"))
    }

    /// Indicates whether an adapter is registered.
    pub fn contains(&self, adapter_id: &str) -> bool {
        self.adapters.contains_key(adapter_id)
    }

    /// Returns all registered adapter identifiers.
    pub fn adapter_ids(&self) -> Vec<&str> {
        self.adapters.keys().map(String::as_str).collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
