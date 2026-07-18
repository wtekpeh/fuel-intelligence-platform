use std::fs;

use crate::identity::IDENTITY_RECORD_SIZE;
use crate::identity::auth::key::ManufacturingKey;
use crate::identity::record::{DecodedIdentity, IdentityAuthentication};

use crate::identity::validation::validate_device_code;
use crate::services::identity::{
    generate_v1_identity, generate_v2_identity, verify_v1_identity, verify_v2_identity,
};

pub fn execute_v1(device_code: &str, output_path: &str) -> Result<(), String> {
    validate_device_code(device_code)?;

    let record = generate_v1_identity(device_code);

    write_identity_file(output_path, &record)?;

    println!("================================");
    println!("ORBI DEVICE IDENTITY GENERATED");
    println!("================================");
    println!("Device Code: {device_code}");
    println!("Identity Format: V1");
    println!("Output File: {output_path}");
    println!("Record Size: {IDENTITY_RECORD_SIZE} bytes");

    let written_record = read_identity_file(output_path)?;

    let decoded = verify_v1_identity(&written_record)?;

    verify_requested_device_code(device_code, &decoded)?;

    print_verified_identity(&decoded);

    Ok(())
}

pub fn execute_v2(device_code: &str, key_file: &str, output_path: &str) -> Result<(), String> {
    validate_device_code(device_code)?;

    let manufacturing_key = ManufacturingKey::load_from_file(key_file)?;

    let record = generate_v2_identity(device_code, &manufacturing_key)?;

    write_identity_file(output_path, &record)?;

    println!("================================");
    println!("ORBI DEVICE IDENTITY GENERATED");
    println!("================================");
    println!("Device Code: {device_code}");
    println!("Identity Format: V2");
    println!("Authentication: HMAC-SHA-256");
    println!("Key File: {key_file}");
    println!(
        "Manufacturing Key Size: {} bytes",
        manufacturing_key.length()
    );
    println!("Output File: {output_path}");
    println!("Record Size: {IDENTITY_RECORD_SIZE} bytes");

    let written_record = read_identity_file(output_path)?;

    let decoded = verify_v2_identity(&written_record, &manufacturing_key)?;

    verify_requested_device_code(device_code, &decoded)?;

    print_verified_identity(&decoded);

    Ok(())
}

fn write_identity_file(
    output_path: &str,
    record: &[u8; IDENTITY_RECORD_SIZE],
) -> Result<(), String> {
    fs::write(output_path, record)
        .map_err(|error| format!("Could not write identity file '{}': {}", output_path, error))
}

fn read_identity_file(output_path: &str) -> Result<Vec<u8>, String> {
    fs::read(output_path).map_err(|error| {
        format!(
            "Could not read generated identity file '{}': {}",
            output_path, error
        )
    })
}

fn verify_requested_device_code(
    requested_device_code: &str,
    decoded: &DecodedIdentity,
) -> Result<(), String> {
    if decoded.device_code != requested_device_code {
        return Err(format!(
            "Stored device code '{}' does not match requested device code '{}'.",
            decoded.device_code, requested_device_code
        ));
    }

    Ok(())
}

fn print_verified_identity(decoded: &DecodedIdentity) {
    println!("================================");
    println!("IDENTITY FILE VERIFIED");
    println!("================================");
    println!("Stored Device Code: {}", decoded.device_code);
    println!("Format Version: {}", decoded.format_version);
    println!("Provisioned: {}", decoded.provisioned);

    match &decoded.authentication {
        IdentityAuthentication::Fnv1a { checksum } => {
            println!("Authentication: FNV-1A CHECKSUM");
            println!("Stored Checksum: 0x{checksum:08X}");
        }

        IdentityAuthentication::HmacSha256 { tag } => {
            println!("Authentication: HMAC-SHA-256");
            println!("Authentication Tag: {}", format_hex(tag));
        }
    }

    println!("Validation: PASSED");
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}
