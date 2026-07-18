use std::fs;
use std::path::Path;

use crate::flash::FlashProvider;
use crate::identity::record::{IdentityAuthentication, decode_identity_record};
use crate::identity::{IDENTITY_FLASH_ADDRESS, IDENTITY_RECORD_SIZE};

pub fn execute(
    flash_provider: &impl FlashProvider,
    port: &str,
    output_path: &str,
) -> Result<(), String> {
    if Path::new(output_path).exists() {
        fs::remove_file(output_path).map_err(|error| {
            format!(
                "Could not remove previous read-back file '{}': {}",
                output_path, error
            )
        })?;
    }

    flash_provider.read_region(
        port,
        IDENTITY_FLASH_ADDRESS,
        IDENTITY_RECORD_SIZE,
        output_path,
    )?;

    let record = fs::read(output_path).map_err(|error| {
        format!(
            "Could not read flash output file '{}': {}",
            output_path, error
        )
    })?;

    if record.len() != IDENTITY_RECORD_SIZE {
        return Err(format!(
            "Flash read returned {} bytes. Expected {} bytes.",
            record.len(),
            IDENTITY_RECORD_SIZE
        ));
    }

    if record.iter().all(|byte| *byte == 0xFF) {
        println!("================================");
        println!("ORBI DEVICE IDENTITY");
        println!("================================");
        println!("Port: {port}");
        println!("Flash Address: 0x{:08X}", IDENTITY_FLASH_ADDRESS);
        println!("Status: BLANK");
        println!("Provisioned: false");
        println!("Identity: NOT PRESENT");

        return Ok(());
    }

    let decoded = decode_identity_record(&record)?;

    println!("================================");
    println!("ORBI DEVICE IDENTITY");
    println!("================================");
    println!("Port: {port}");
    println!("Flash Address: 0x{:08X}", IDENTITY_FLASH_ADDRESS);
    println!("Status: PROVISIONED");
    println!("Device Code: {}", decoded.device_code);
    println!("Format Version: {}", decoded.format_version);
    println!("Provisioned: {}", decoded.provisioned);

    match decoded.authentication {
        IdentityAuthentication::Fnv1a { checksum } => {
            println!("Authentication: FNV-1A CHECKSUM");
            println!("Checksum: 0x{checksum:08X}");
        }

        IdentityAuthentication::HmacSha256 { tag } => {
            println!("Authentication: HMAC-SHA-256");
            println!("Authentication Tag: {}", format_hex(&tag));
        }
    }

    println!("Identity: VALID");

    Ok(())
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}
