use std::fmt;

/// Identifies a supported sensor calibration category.
///
/// The database may continue storing calibration types as text,
/// but application code should use this enum rather than passing
/// arbitrary string values between services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalibrationType {
    Imu,
    Fuel,
}

impl CalibrationType {
    /// Returns the canonical value stored in PostgreSQL.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Imu => "imu",
            Self::Fuel => "fuel",
        }
    }
}

impl fmt::Display for CalibrationType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::CalibrationType;

    #[test]
    fn calibration_types_use_canonical_database_values() {
        assert_eq!(CalibrationType::Imu.as_str(), "imu");
        assert_eq!(CalibrationType::Imu.to_string(), "imu");

        assert_eq!(CalibrationType::Fuel.as_str(), "fuel");
        assert_eq!(CalibrationType::Fuel.to_string(), "fuel");
    }
}
