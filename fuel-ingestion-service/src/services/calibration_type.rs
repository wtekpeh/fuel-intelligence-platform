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
    /// Parses an external calibration type into a supported domain value.
    ///
    /// External callers may use different casing or surrounding whitespace,
    /// but application code and PostgreSQL always use the canonical lowercase
    /// representation returned by `as_str()`.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "imu" => Some(Self::Imu),
            "fuel" => Some(Self::Fuel),
            _ => None,
        }
    }

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

    #[test]
    fn supported_calibration_types_are_parsed() {
        assert_eq!(CalibrationType::parse("imu"), Some(CalibrationType::Imu));

        assert_eq!(CalibrationType::parse("IMU"), Some(CalibrationType::Imu));

        assert_eq!(
            CalibrationType::parse(" fuel "),
            Some(CalibrationType::Fuel)
        );

        assert_eq!(CalibrationType::parse("FUEL"), Some(CalibrationType::Fuel));
    }

    #[test]
    fn unsupported_calibration_type_is_rejected() {
        assert_eq!(CalibrationType::parse("temperature"), None);
        assert_eq!(CalibrationType::parse(""), None);
        assert_eq!(CalibrationType::parse("   "), None);
    }
}
