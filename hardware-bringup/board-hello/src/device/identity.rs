pub struct DeviceIdentity {
    pub device_code: &'static str,
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

pub const DEVICE_IDENTITY: DeviceIdentity = DeviceIdentity {
    device_code: "ORBI-GPS-002",
    firmware_version: "0.1.0",
    product_code: "ORBI-GPS-LITE",
    hardware_profile_code: "GPS_ONLY",
    capabilities: DeviceCapabilities {
        gps: true,
        fuel: false,
        vibration: false,
        kill_switch: false,
    },
};
