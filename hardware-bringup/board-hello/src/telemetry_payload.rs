use heapless::String;

pub struct GpsReading<'a> {
    pub device_id: &'static str,
    pub timestamp: &'a str,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub heading: f64,
}

pub fn build_gps_only_payload(reading: &GpsReading<'_>) -> String<1024> {
    let mut payload = String::<1024>::new();

    let _ = core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"synced_at\":\"{}\",\
                \"readings\":[{{\
                    \"device_id\":\"{}\",\
                    \"timestamp\":\"{}\",\
                    \"fuel_level_litres\":0.0,\
                    \"fuel_level_percentage\":0.0,\
                    \"latitude\":{},\
                    \"longitude\":{},\
                    \"vibration_level\":0.0,\
                    \"motion_detected\":false,\
                    \"simulation_mode\":\"real_gps_only\"\
                }}]\
            }}",
            reading.device_id,
            reading.timestamp,
            reading.device_id,
            reading.timestamp,
            reading.latitude,
            reading.longitude,
        ),
    );

    payload
}
