use heapless::String;

pub const MAX_DEVICE_CODE_LENGTH: usize = 32;

pub struct FirmwareIdentity {
    pub firmware_version: &'static str,
    pub product_code: &'static str,
    pub hardware_profile_code: &'static str,
    pub capabilities: DeviceCapabilities,
}

pub struct DeviceCapabilities {
    pub gps: bool,
    pub fuel: bool,
    pub vibration: bool,
    pub kill_switch: bool,
}

pub struct RuntimeDeviceIdentity {
    device_code: String<MAX_DEVICE_CODE_LENGTH>,
    provisioned: bool,
}

impl RuntimeDeviceIdentity {
    pub fn from_device_code(device_code: &str, provisioned: bool) -> Option<Self> {
        if device_code.is_empty() {
            return None;
        }

        let mut stored_device_code = String::<MAX_DEVICE_CODE_LENGTH>::new();

        stored_device_code.push_str(device_code).ok()?;

        Some(Self {
            device_code: stored_device_code,
            provisioned,
        })
    }

    pub fn device_code(&self) -> &str {
        self.device_code.as_str()
    }

    pub fn is_provisioned(&self) -> bool {
        self.provisioned
    }
}

pub const FIRMWARE_IDENTITY: FirmwareIdentity = FirmwareIdentity {
    firmware_version: "0.2.0",
    product_code: "ORBI-GPS-LITE",
    hardware_profile_code: "GPS_ONLY",
    capabilities: DeviceCapabilities {
        gps: true,
        fuel: false,
        vibration: false,
        kill_switch: false,
    },
};

pub fn load_runtime_identity() -> RuntimeDeviceIdentity {
    // Temporary development fallback.
    //
    // The next implementation step will replace this value with a device
    // code loaded from persistent ESP32 flash storage.
    RuntimeDeviceIdentity::from_device_code("ORBI-GPS-002", false)
        .expect("development device code must be valid")
}
