/*
 * Represents one normalized telemetry measurement cycle.
 *
 * This structure contains physical measurements only.
 * It must not contain operational intelligence such as:
 *
 * - movement classification
 * - vibration severity
 * - impact detection
 * - harsh braking detection
 * - alert decisions
 *
 * Those responsibilities belong to the ORBI backend.
 */
pub struct TelemetryRecord<'a> {
    /*
     * Device and measurement identity.
     */
    pub device_id: &'a str,
    pub timestamp: &'a str,

    /*
     * GNSS measurements.
     */
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub heading: f64,

    /*
     * Fuel measurements.
     *
     * These fields remain in the shared telemetry structure so the existing
     * telemetry pipeline is not disrupted. They will be connected to the
     * physical fuel sensor during the later fuel-sensor phase.
     */
    pub fuel_level_litres: f32,
    pub fuel_level_percentage: f32,

    /*
     * MPU6050 accelerometer measurements.
     *
     * Units: gravitational acceleration, g.
     */
    pub accel_x_g: f32,
    pub accel_y_g: f32,
    pub accel_z_g: f32,

    /*
     * MPU6050 gyroscope measurements.
     *
     * Units: degrees per second.
     */
    pub gyro_x_dps: f32,
    pub gyro_y_dps: f32,
    pub gyro_z_dps: f32,

    /*
     * MPU6050 internal temperature measurement.
     *
     * Unit: degrees Celsius.
     */
    pub imu_temperature_c: f32,

    /*
     * Indicates whether the current runtime is using simulated or physical
     * measurements.
     *
     * This is metadata, not sensor intelligence.
     */
    pub simulation_mode: &'a str,
}
