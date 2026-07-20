use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use esp_println::println;

/*
 * Default MPU6050 I²C address.
 *
 * AD0 LOW  -> 0x68
 * AD0 HIGH -> 0x69
 */
const MPU6050_ADDRESS: u8 = 0x68;

/*
 * MPU6050 register addresses.
 */
const WHO_AM_I_REGISTER: u8 = 0x75;
const POWER_MANAGEMENT_1_REGISTER: u8 = 0x6B;

/*
 * The MPU6050 WHO_AM_I register should return 0x68.
 */
const EXPECTED_WHO_AM_I_VALUE: u8 = 0x68;

//
const ACCEL_XOUT_H: u8 = 0x3B;
const ACCEL_YOUT_H: u8 = 0x3D;
const ACCEL_ZOUT_H: u8 = 0x3F;

const TEMP_OUT_H: u8 = 0x41;

const GYRO_XOUT_H: u8 = 0x43;
const GYRO_YOUT_H: u8 = 0x45;
const GYRO_ZOUT_H: u8 = 0x47;

const CONFIG_REGISTER: u8 = 0x1A;
const GYROSCOPE_CONFIG_REGISTER: u8 = 0x1B;
const ACCELEROMETER_CONFIG_REGISTER: u8 = 0x1C;

/*
 * MPU6050 configuration values.
 *
 * CONFIG = 0x03
 *   Digital low-pass filter bandwidth:
 *   approximately 44 Hz for accelerometer
 *   approximately 42 Hz for gyroscope.
 *
 * GYRO_CONFIG = 0x00
 *   Full-scale range: ±250 degrees per second.
 *
 * ACCEL_CONFIG = 0x00
 *   Full-scale range: ±2 g.
 */
const DIGITAL_LOW_PASS_FILTER_CONFIG: u8 = 0x03;
const GYROSCOPE_RANGE_250_DPS: u8 = 0x00;
const ACCELEROMETER_RANGE_2G: u8 = 0x00;

/*
 * Conversion factors for the configured measurement ranges.
 *
 * Accelerometer:
 * ±2 g gives 16,384 raw units for every 1 g.
 *
 * Gyroscope:
 * ±250 degrees/second gives 131 raw units for every 1 degree/second.
 */
const ACCELEROMETER_SENSITIVITY_LSB_PER_G: f32 = 16_384.0;
const GYROSCOPE_SENSITIVITY_LSB_PER_DPS: f32 = 131.0;

/*
 * MPU6050 temperature conversion constants.
 *
 * temperature_c = raw_temperature / 340.0 + 36.53
 */
const TEMPERATURE_SENSITIVITY: f32 = 340.0;
const TEMPERATURE_OFFSET_C: f32 = 36.53;

pub struct RawImuData {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,

    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,

    pub temperature: i16,
}

/*
 * MPU6050 readings converted into physical units.
 *
 * Accelerometer values are expressed in g.
 * Gyroscope values are expressed in degrees per second.
 * Temperature is expressed in degrees Celsius.
 */
pub struct ImuData {
    pub accel_x_g: f32,
    pub accel_y_g: f32,
    pub accel_z_g: f32,

    pub gyro_x_dps: f32,
    pub gyro_y_dps: f32,
    pub gyro_z_dps: f32,

    pub temperature_c: f32,
}

/*
 * Write one byte into an MPU6050 register.
 */
fn write_register(i2c: &mut I2c<'_, Blocking>, register: u8, value: u8) -> bool {
    let write_buffer = [register, value];

    let result = i2c.write(MPU6050_ADDRESS, &write_buffer);

    if result.is_ok() {
        true
    } else {
        println!(
            "ERROR: Failed to write value 0x{:02X} to MPU6050 register 0x{:02X}.",
            value, register
        );

        false
    }
}

/*
 * Read one byte from an MPU6050 register.
 */
fn read_register(i2c: &mut I2c<'_, Blocking>, register: u8) -> Option<u8> {
    let register_buffer = [register];
    let mut read_buffer = [0u8; 1];

    let result = i2c.write_read(MPU6050_ADDRESS, &register_buffer, &mut read_buffer);

    if result.is_ok() {
        Some(read_buffer[0])
    } else {
        println!("ERROR: Failed to read MPU6050 register 0x{:02X}.", register);

        None
    }
}

fn read_i16(i2c: &mut I2c<'_, Blocking>, register: u8) -> Option<i16> {
    let register_buffer = [register];
    let mut read_buffer = [0u8; 2];

    let result = i2c.write_read(MPU6050_ADDRESS, &register_buffer, &mut read_buffer);

    if result.is_err() {
        println!(
            "ERROR: Failed to read register pair starting at 0x{:02X}",
            register
        );

        return None;
    }

    let value = i16::from_be_bytes([read_buffer[0], read_buffer[1]]);

    Some(value)
}

pub fn read_raw_data(i2c: &mut I2c<'_, Blocking>) -> Option<RawImuData> {
    Some(RawImuData {
        accel_x: read_i16(i2c, ACCEL_XOUT_H)?,
        accel_y: read_i16(i2c, ACCEL_YOUT_H)?,
        accel_z: read_i16(i2c, ACCEL_ZOUT_H)?,

        temperature: read_i16(i2c, TEMP_OUT_H)?,

        gyro_x: read_i16(i2c, GYRO_XOUT_H)?,
        gyro_y: read_i16(i2c, GYRO_YOUT_H)?,
        gyro_z: read_i16(i2c, GYRO_ZOUT_H)?,
    })
}

/*
 * Convert raw MPU6050 register values into physical units.
 */
pub fn convert_raw_data(raw_data: &RawImuData) -> ImuData {
    ImuData {
        accel_x_g: raw_data.accel_x as f32 / ACCELEROMETER_SENSITIVITY_LSB_PER_G,

        accel_y_g: raw_data.accel_y as f32 / ACCELEROMETER_SENSITIVITY_LSB_PER_G,

        accel_z_g: raw_data.accel_z as f32 / ACCELEROMETER_SENSITIVITY_LSB_PER_G,

        gyro_x_dps: raw_data.gyro_x as f32 / GYROSCOPE_SENSITIVITY_LSB_PER_DPS,

        gyro_y_dps: raw_data.gyro_y as f32 / GYROSCOPE_SENSITIVITY_LSB_PER_DPS,

        gyro_z_dps: raw_data.gyro_z as f32 / GYROSCOPE_SENSITIVITY_LSB_PER_DPS,

        temperature_c: raw_data.temperature as f32 / TEMPERATURE_SENSITIVITY + TEMPERATURE_OFFSET_C,
    }
}

/*
 * Read the MPU6050 and return values in physical units.
 */
pub fn read_imu_data(i2c: &mut I2c<'_, Blocking>) -> Option<ImuData> {
    let raw_data = read_raw_data(i2c)?;

    let imu_data = convert_raw_data(&raw_data);

    Some(imu_data)
}

pub fn print_raw_data(i2c: &mut I2c<'_, Blocking>) {
    if let Some(data) = read_raw_data(i2c) {
        println!("--------------------------------");

        println!(
            "ACCEL  X:{}  Y:{}  Z:{}",
            data.accel_x, data.accel_y, data.accel_z
        );

        println!(
            "GYRO   X:{}  Y:{}  Z:{}",
            data.gyro_x, data.gyro_y, data.gyro_z
        );

        println!("TEMP   {}", data.temperature);
    }
}

/*
 * Print MPU6050 readings in physical engineering units.
 */
pub fn print_imu_data(i2c: &mut I2c<'_, Blocking>) {
    let imu_data = read_imu_data(i2c);

    match imu_data {
        Some(data) => {
            println!("--------------------------------");

            println!(
                "ACCEL  X:{:.3} g  Y:{:.3} g  Z:{:.3} g",
                data.accel_x_g, data.accel_y_g, data.accel_z_g
            );

            println!(
                "GYRO   X:{:.2} dps  Y:{:.2} dps  Z:{:.2} dps",
                data.gyro_x_dps, data.gyro_y_dps, data.gyro_z_dps
            );

            println!("TEMP   {:.2} C", data.temperature_c);
        }

        None => {
            println!("ERROR: Failed to obtain converted MPU6050 readings.");
        }
    }
}

/*
 * Read and verify the MPU6050 identity register.
 */
pub fn verify_identity(i2c: &mut I2c<'_, Blocking>) -> bool {
    println!("Reading MPU6050 WHO_AM_I register...");

    let identity = read_register(i2c, WHO_AM_I_REGISTER);

    match identity {
        Some(value) => {
            println!("WHO_AM_I returned: 0x{:02X}", value);

            if value == EXPECTED_WHO_AM_I_VALUE {
                println!("MPU6050 identity verified successfully.");

                true
            } else {
                println!(
                    "ERROR: Unexpected WHO_AM_I value. Expected 0x{:02X}.",
                    EXPECTED_WHO_AM_I_VALUE
                );

                false
            }
        }

        None => {
            println!("ERROR: MPU6050 identity could not be read.");

            false
        }
    }
}

/*
 * Wake the MPU6050.
 *
 * The MPU6050 starts in sleep mode after power-on.
 * Writing 0x00 to PWR_MGMT_1 clears the sleep bit.
 */
pub fn wake_up(i2c: &mut I2c<'_, Blocking>) -> bool {
    println!("Waking MPU6050 from sleep mode...");

    let wake_succeeded = write_register(i2c, POWER_MANAGEMENT_1_REGISTER, 0x00);

    if wake_succeeded {
        println!("MPU6050 wake command completed.");
    }

    wake_succeeded
}

/*
 * Configure the MPU6050 into a known operating state.
 *
 * Accelerometer range:
 * ±2 g
 *
 * Gyroscope range:
 * ±250 degrees per second
 *
 * Digital low-pass filter:
 * Approximately 44 Hz accelerometer bandwidth
 * Approximately 42 Hz gyroscope bandwidth
 */
pub fn configure(i2c: &mut I2c<'_, Blocking>) -> bool {
    println!("Configuring MPU6050...");

    let filter_configured = write_register(i2c, CONFIG_REGISTER, DIGITAL_LOW_PASS_FILTER_CONFIG);

    if !filter_configured {
        println!("ERROR: Failed to configure MPU6050 low-pass filter.");

        return false;
    }

    let gyroscope_configured =
        write_register(i2c, GYROSCOPE_CONFIG_REGISTER, GYROSCOPE_RANGE_250_DPS);

    if !gyroscope_configured {
        println!("ERROR: Failed to configure MPU6050 gyroscope range.");

        return false;
    }

    let accelerometer_configured =
        write_register(i2c, ACCELEROMETER_CONFIG_REGISTER, ACCELEROMETER_RANGE_2G);

    if !accelerometer_configured {
        println!("ERROR: Failed to configure MPU6050 accelerometer range.");

        return false;
    }

    println!("MPU6050 configuration completed.");
    println!("Accelerometer range: +/- 2 g");
    println!("Gyroscope range: +/- 250 degrees/second");
    println!("Digital low-pass filter enabled.");

    true
}

/*
 * Perform the MPU6050 bring-up sequence.
 *
 * 1. Verify the WHO_AM_I register.
 * 2. Wake the device from sleep mode.
 * 3. Configure the measurement ranges and filter.
 */
pub fn initialize(i2c: &mut I2c<'_, Blocking>) -> bool {
    println!("========================");
    println!("ORBI MPU6050 INITIALIZATION");
    println!("========================");

    let identity_verified = verify_identity(i2c);

    if !identity_verified {
        println!("MPU6050 initialization stopped because identity verification failed.");

        return false;
    }

    let wake_succeeded = wake_up(i2c);

    if !wake_succeeded {
        println!("MPU6050 initialization stopped because wake-up failed.");

        return false;
    }

    let configuration_succeeded = configure(i2c);

    if !configuration_succeeded {
        println!("MPU6050 initialization stopped because configuration failed.");

        return false;
    }

    println!("MPU6050 initialization successful.");

    true
}
