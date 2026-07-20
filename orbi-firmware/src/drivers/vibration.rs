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
 * Perform the first MPU6050 bring-up sequence.
 *
 * For now this only:
 *
 * 1. Verifies WHO_AM_I.
 * 2. Wakes the sensor.
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

    println!("MPU6050 initialization successful.");

    true
}
