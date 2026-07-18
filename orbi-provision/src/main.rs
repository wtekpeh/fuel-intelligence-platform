use std::env;
use std::fs;
use std::process;

const IDENTITY_RECORD_SIZE: usize = 64;
const DEVICE_CODE_CAPACITY: usize = 32;

const MAGIC: [u8; 4] = *b"ORBI";
const FORMAT_VERSION: u8 = 1;
const PROVISIONED_FLAG: u8 = 0x01;

const DEVICE_CODE_OFFSET: usize = 8;
const CHECKSUM_OFFSET: usize = 40;

fn main() {
    let device_code = read_device_code();

    if let Err(message) = validate_device_code(&device_code) {
        eprintln!("[ERROR] {message}");
        process::exit(1);
    }

    let record = build_identity_record(&device_code);
    let checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    let output_path = "identity.bin";

    if let Err(error) = fs::write(output_path, record) {
        eprintln!(
            "[ERROR] Could not write identity file '{}': {}",
            output_path, error
        );

        process::exit(1);
    }

    println!("================================");
    println!("ORBI DEVICE IDENTITY GENERATED");
    println!("================================");
    println!("Device Code: {}", device_code);
    println!("Format Version: {}", FORMAT_VERSION);
    println!("Provisioned: true");
    println!("Checksum: 0x{:08X}", checksum);
    println!("Output File: {}", output_path);
    println!("Record Size: {} bytes", IDENTITY_RECORD_SIZE);

    verify_written_record(output_path, &device_code);
}

fn read_device_code() -> String {
    let mut arguments = env::args();

    let executable_name = arguments
        .next()
        .unwrap_or_else(|| String::from("orbi-provision"));

    let Some(device_code) = arguments.next() else {
        eprintln!("Usage:");
        eprintln!("  {executable_name} <DEVICE_CODE>");
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {executable_name} ORBI-A100-000004");

        process::exit(1);
    };

    if arguments.next().is_some() {
        eprintln!("[ERROR] Only one device code may be supplied.");
        process::exit(1);
    }

    device_code
}

fn validate_device_code(device_code: &str) -> Result<(), &'static str> {
    if device_code.is_empty() {
        return Err("Device code cannot be empty.");
    }

    if device_code.len() > DEVICE_CODE_CAPACITY {
        return Err("Device code cannot exceed 32 characters.");
    }

    if !device_code.is_ascii() {
        return Err("Device code must contain ASCII characters only.");
    }

    if !device_code
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err("Device code may contain only uppercase letters, numbers, and hyphens.");
    }

    Ok(())
}

fn build_identity_record(device_code: &str) -> [u8; IDENTITY_RECORD_SIZE] {
    let device_code_bytes = device_code.as_bytes();

    let mut record = [0xFFu8; IDENTITY_RECORD_SIZE];

    record[0..4].copy_from_slice(&MAGIC);
    record[4] = FORMAT_VERSION;
    record[5] = PROVISIONED_FLAG;
    record[6] = device_code_bytes.len() as u8;
    record[7] = 0xFF;

    let device_code_end = DEVICE_CODE_OFFSET + device_code_bytes.len();

    record[DEVICE_CODE_OFFSET..device_code_end].copy_from_slice(device_code_bytes);

    let checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    record[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].copy_from_slice(&checksum.to_le_bytes());

    record
}

fn verify_written_record(output_path: &str, expected_device_code: &str) {
    let record = match fs::read(output_path) {
        Ok(record) => record,

        Err(error) => {
            eprintln!(
                "[ERROR] Could not read generated identity file '{}': {}",
                output_path, error
            );

            process::exit(1);
        }
    };

    if record.len() != IDENTITY_RECORD_SIZE {
        eprintln!(
            "[ERROR] Identity record has invalid size: {} bytes",
            record.len()
        );

        process::exit(1);
    }

    if record[0..4] != MAGIC {
        eprintln!("[ERROR] Identity record has an invalid magic value.");
        process::exit(1);
    }

    if record[4] != FORMAT_VERSION {
        eprintln!(
            "[ERROR] Identity record has unsupported format version: {}",
            record[4]
        );

        process::exit(1);
    }

    if record[5] & PROVISIONED_FLAG == 0 {
        eprintln!("[ERROR] Identity record is not marked as provisioned.");
        process::exit(1);
    }

    let device_code_length = record[6] as usize;

    if device_code_length == 0 || device_code_length > DEVICE_CODE_CAPACITY {
        eprintln!(
            "[ERROR] Identity record contains invalid device-code length: {}",
            device_code_length
        );

        process::exit(1);
    }

    let device_code_end = DEVICE_CODE_OFFSET + device_code_length;

    let stored_device_code = match std::str::from_utf8(&record[DEVICE_CODE_OFFSET..device_code_end])
    {
        Ok(device_code) => device_code,

        Err(_) => {
            eprintln!("[ERROR] Stored device code is not valid UTF-8.");
            process::exit(1);
        }
    };

    let expected_checksum = u32::from_le_bytes([
        record[CHECKSUM_OFFSET],
        record[CHECKSUM_OFFSET + 1],
        record[CHECKSUM_OFFSET + 2],
        record[CHECKSUM_OFFSET + 3],
    ]);

    let calculated_checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    if expected_checksum != calculated_checksum {
        eprintln!("[ERROR] Identity checksum verification failed.");
        eprintln!("Expected checksum: 0x{:08X}", expected_checksum);
        eprintln!("Calculated checksum: 0x{:08X}", calculated_checksum);

        process::exit(1);
    }

    if stored_device_code != expected_device_code {
        eprintln!(
            "[ERROR] Stored device code '{}' does not match '{}'.",
            stored_device_code, expected_device_code
        );

        process::exit(1);
    }

    println!("================================");
    println!("IDENTITY FILE VERIFIED");
    println!("================================");
    println!("Stored Device Code: {}", stored_device_code);
    println!("Stored Checksum: 0x{:08X}", expected_checksum);
    println!("Validation: PASSED");
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
