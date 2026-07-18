use embedded_storage::ReadStorage;
use esp_hal::peripherals::FLASH;
use esp_println::println;
use esp_storage::FlashStorage;

use super::identity::RuntimeDeviceIdentity;

pub const ORBI_CONFIG_OFFSET: u32 = 0x001F_0000;
pub const ORBI_CONFIG_SIZE: u32 = 0x0001_0000;

const IDENTITY_RECORD_SIZE: usize = 64;
const MAGIC: [u8; 4] = *b"ORBI";
const FORMAT_VERSION: u8 = 1;
const PROVISIONED_FLAG: u8 = 0x01;

const DEVICE_CODE_OFFSET: usize = 8;
const DEVICE_CODE_CAPACITY: usize = 32;
const CHECKSUM_OFFSET: usize = 40;

pub fn load_runtime_identity_from_flash(flash: FLASH<'static>) -> Option<RuntimeDeviceIdentity> {
    println!("========================");
    println!("ORBI CONFIGURATION STORAGE");
    println!("========================");

    let mut storage = FlashStorage::new(flash);
    let mut record = [0u8; IDENTITY_RECORD_SIZE];

    if let Err(error) = storage.read(ORBI_CONFIG_OFFSET, &mut record) {
        println!("Failed to read ORBI configuration partition: {:?}", error);

        return None;
    }

    println!("Configuration offset: 0x{:08X}", ORBI_CONFIG_OFFSET);
    println!("Configuration size: {} bytes", ORBI_CONFIG_SIZE);
    println!("Identity record size: {} bytes", IDENTITY_RECORD_SIZE);

    if record.iter().all(|byte| *byte == 0xFF) {
        println!("Partition erased: true");
        println!("No provisioned device identity found.");

        return None;
    }

    println!("Partition erased: false");
    println!("Existing configuration data found.");

    parse_identity_record(&record)
}

fn parse_identity_record(record: &[u8; IDENTITY_RECORD_SIZE]) -> Option<RuntimeDeviceIdentity> {
    if record[0..4] != MAGIC {
        println!("Invalid identity magic value.");
        println!("Stored configuration will not be used.");

        return None;
    }

    let stored_version = record[4];

    if stored_version != FORMAT_VERSION {
        println!("Unsupported identity format version: {}", stored_version);

        return None;
    }

    let flags = record[5];

    if flags & PROVISIONED_FLAG == 0 {
        println!("Identity record is not marked as provisioned.");

        return None;
    }

    let device_code_length = record[6] as usize;

    if device_code_length == 0 || device_code_length > DEVICE_CODE_CAPACITY {
        println!(
            "Invalid provisioned device-code length: {}",
            device_code_length
        );

        return None;
    }

    let expected_checksum = u32::from_le_bytes([
        record[CHECKSUM_OFFSET],
        record[CHECKSUM_OFFSET + 1],
        record[CHECKSUM_OFFSET + 2],
        record[CHECKSUM_OFFSET + 3],
    ]);

    let calculated_checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    if expected_checksum != calculated_checksum {
        println!("Identity checksum validation failed.");
        println!("Expected checksum: 0x{:08X}", expected_checksum);
        println!("Calculated checksum: 0x{:08X}", calculated_checksum);

        return None;
    }

    let device_code_end = DEVICE_CODE_OFFSET + device_code_length;

    let device_code_bytes = &record[DEVICE_CODE_OFFSET..device_code_end];

    let device_code = match core::str::from_utf8(device_code_bytes) {
        Ok(device_code) => device_code,

        Err(_) => {
            println!("Provisioned device code is not valid UTF-8.");

            return None;
        }
    };

    let runtime_identity = match RuntimeDeviceIdentity::from_device_code(device_code, true) {
        Some(identity) => identity,

        None => {
            println!("Provisioned device code is invalid.");

            return None;
        }
    };

    println!("Provisioned identity loaded successfully.");
    println!("Stored Device Code: {}", runtime_identity.device_code());

    Some(runtime_identity)
}

fn calculate_checksum(data: &[u8]) -> u32 {
    const FNV_OFFSET_BASIS: u32 = 0x811C_9DC5;
    const FNV_PRIME: u32 = 0x0100_0193;

    let mut checksum = FNV_OFFSET_BASIS;

    for byte in data {
        checksum ^= *byte as u32;
        checksum = checksum.wrapping_mul(FNV_PRIME);
    }

    checksum
}
