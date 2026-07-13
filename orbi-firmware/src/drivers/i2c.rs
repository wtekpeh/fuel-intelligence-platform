use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use esp_println::println;

pub fn print_scan_banner() {
    println!("========================");
    println!("QMI8658 I2C SCAN TEST");
    println!("========================");
}

pub fn scan_i2c_bus(i2c: &mut I2c<'_, Blocking>) {
    println!("Scanning I2C bus...");

    let mut found_count = 0;

    for address in 0x08u8..=0x77u8 {
        let result = i2c.write(address, &[]);

        if result.is_ok() {
            found_count += 1;
            println!("Found I2C device at address: 0x{:02X}", address);
        }
    }

    if found_count == 0 {
        println!("No I2C devices found.");
    } else {
        println!("I2C scan complete. Devices found: {}", found_count);
    }
}
