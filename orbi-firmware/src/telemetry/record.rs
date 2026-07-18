pub struct TelemetryRecord<'a> {
    pub device_id: &'a str,
    pub timestamp: &'a str,

    pub latitude: f64,
    pub longitude: f64,

    pub fuel_level_litres: f32,
    pub fuel_level_percentage: f32,

    pub vibration_level: f32,
    pub motion_detected: bool,

    pub speed: f64,
    pub heading: f64,

    pub simulation_mode: &'a str,
}
