#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryStatus {
    Assembled,
    Programmed,
    Tested,
    ReadyForDeployment,
    Provisioned,
    Retired,
}

impl InventoryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assembled => "ASSEMBLED",
            Self::Programmed => "PROGRAMMED",
            Self::Tested => "TESTED",
            Self::ReadyForDeployment => "READY_FOR_DEPLOYMENT",
            Self::Provisioned => "PROVISIONED",
            Self::Retired => "RETIRED",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "ASSEMBLED" => Some(Self::Assembled),
            "PROGRAMMED" => Some(Self::Programmed),
            "TESTED" => Some(Self::Tested),
            "READY_FOR_DEPLOYMENT" => Some(Self::ReadyForDeployment),
            "PROVISIONED" => Some(Self::Provisioned),
            "RETIRED" => Some(Self::Retired),
            _ => None,
        }
    }

    pub fn can_transition_to(self, next: InventoryStatus) -> bool {
        matches!(
            (self, next),
            (Self::Assembled, Self::Programmed)
                | (Self::Programmed, Self::Tested)
                | (Self::Tested, Self::ReadyForDeployment)
                | (Self::ReadyForDeployment, Self::Provisioned)
                | (Self::Provisioned, Self::Retired)
        )
    }
}
