use crate::flash::FlashProvider;
use crate::identity::auth::key::ManufacturingKey;
use crate::identity::record::IdentityAuthentication;
use crate::identity::validation::validate_device_code;
use crate::services::provision::provision_v2;

pub fn execute_v2(
    flash_provider: &impl FlashProvider,
    port: &str,
    device_code: &str,
    key_file: &str,
) -> Result<(), String> {
    validate_device_code(device_code)?;

    let manufacturing_key = ManufacturingKey::load_from_file(key_file)?;

    let decoded = provision_v2(flash_provider, port, device_code, &manufacturing_key)?;

    println!("================================");
    println!("ORBI DEVICE PROVISIONED");
    println!("================================");
    println!("Port: {port}");
    println!("Device Code: {}", decoded.device_code);
    println!("Format Version: {}", decoded.format_version);
    println!("Provisioned: {}", decoded.provisioned);

    match decoded.authentication {
        IdentityAuthentication::HmacSha256 { tag } => {
            println!("Authentication: HMAC-SHA-256");
            println!("Authentication Tag: {}", format_hex(&tag));
        }

        IdentityAuthentication::Fnv1a { checksum } => {
            return Err(format!(
                "Provisioning unexpectedly returned a V1 identity with checksum 0x{checksum:08X}."
            ));
        }
    }

    println!("Flash Verification: PASSED");
    println!("HMAC Verification: PASSED");
    println!("Provisioning Result: SUCCESS");

    Ok(())
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}
