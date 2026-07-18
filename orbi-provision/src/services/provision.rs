use std::fs;
use std::path::PathBuf;
use std::process;

use crate::flash::FlashProvider;
use crate::identity::auth::key::ManufacturingKey;
use crate::identity::record::DecodedIdentity;
use crate::identity::{IDENTITY_FLASH_ADDRESS, IDENTITY_RECORD_SIZE};
use crate::services::identity::{generate_v2_identity, verify_v2_identity};

pub fn provision_v2(
    flash_provider: &impl FlashProvider,
    port: &str,
    device_code: &str,
    manufacturing_key: &ManufacturingKey,
) -> Result<DecodedIdentity, String> {
    let temporary_files = ProvisionTemporaryFiles::new();

    remove_file_if_present(&temporary_files.precheck)?;
    remove_file_if_present(&temporary_files.identity)?;
    remove_file_if_present(&temporary_files.read_back)?;

    println!("================================");
    println!("ORBI V2 PROVISIONING");
    println!("================================");
    println!("Port: {port}");
    println!("Device Code: {device_code}");
    println!("Flash Address: 0x{:08X}", IDENTITY_FLASH_ADDRESS);

    /*
     * Read the identity region before writing anything.
     */
    flash_provider.read_region(
        port,
        IDENTITY_FLASH_ADDRESS,
        IDENTITY_RECORD_SIZE,
        path_as_string(&temporary_files.precheck)?,
    )?;

    let current_record = read_exact_identity_record(&temporary_files.precheck)?;

    /*
     * Protect an existing device identity.
     *
     * Provisioning is allowed only when every byte in the identity
     * region is erased to 0xFF.
     */
    if !is_blank_identity(&current_record) {
        return Err(String::from(
            "The device identity region is not blank. Provisioning was aborted to protect the existing identity.",
        ));
    }

    println!("Identity Region: BLANK");
    println!("Overwrite Protection: PASSED");

    /*
     * Generate the authenticated V2 identity entirely in memory.
     */
    let generated_record = generate_v2_identity(device_code, manufacturing_key)?;

    fs::write(&temporary_files.identity, generated_record).map_err(|error| {
        format!(
            "Could not create temporary provisioning file '{}': {}",
            temporary_files.identity.display(),
            error
        )
    })?;

    /*
     * Write the generated 64-byte identity to the board.
     */
    flash_provider.write_region(
        port,
        IDENTITY_FLASH_ADDRESS,
        path_as_string(&temporary_files.identity)?,
    )?;

    /*
     * Read the same region back after writing.
     */
    flash_provider.read_region(
        port,
        IDENTITY_FLASH_ADDRESS,
        IDENTITY_RECORD_SIZE,
        path_as_string(&temporary_files.read_back)?,
    )?;

    let written_record = read_exact_identity_record(&temporary_files.read_back)?;

    /*
     * Ensure the bytes on the board exactly match the bytes generated
     * by the provisioning service.
     */
    if written_record != generated_record {
        return Err(String::from(
            "Flash verification failed: the read-back identity does not exactly match the generated identity.",
        ));
    }

    /*
     * Cryptographically verify the record using the manufacturing key.
     */
    let decoded = verify_v2_identity(&written_record, manufacturing_key)?;

    if decoded.device_code != device_code {
        return Err(format!(
            "Verified device code '{}' does not match requested device code '{}'.",
            decoded.device_code, device_code
        ));
    }

    if !decoded.provisioned {
        return Err(String::from(
            "The written identity is not marked as provisioned.",
        ));
    }

    Ok(decoded)
}

fn is_blank_identity(record: &[u8; IDENTITY_RECORD_SIZE]) -> bool {
    record.iter().all(|byte| *byte == 0xFF)
}

fn read_exact_identity_record(path: &PathBuf) -> Result<[u8; IDENTITY_RECORD_SIZE], String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "Could not read identity file '{}': {}",
            path.display(),
            error
        )
    })?;

    if bytes.len() != IDENTITY_RECORD_SIZE {
        return Err(format!(
            "Identity file '{}' contains {} bytes. Expected {} bytes.",
            path.display(),
            bytes.len(),
            IDENTITY_RECORD_SIZE
        ));
    }

    let mut record = [0_u8; IDENTITY_RECORD_SIZE];
    record.copy_from_slice(&bytes);

    Ok(record)
}

fn remove_file_if_present(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).map_err(|error| {
        format!(
            "Could not remove previous temporary file '{}': {}",
            path.display(),
            error
        )
    })
}

fn path_as_string(path: &PathBuf) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("Temporary path '{}' is not valid UTF-8.", path.display()))
}

struct ProvisionTemporaryFiles {
    precheck: PathBuf,
    identity: PathBuf,
    read_back: PathBuf,
}

impl ProvisionTemporaryFiles {
    fn new() -> Self {
        let process_id = process::id();
        let temporary_directory = std::env::temp_dir();

        Self {
            precheck: temporary_directory.join(format!("orbi-provision-{process_id}-precheck.bin")),
            identity: temporary_directory.join(format!("orbi-provision-{process_id}-identity.bin")),
            read_back: temporary_directory
                .join(format!("orbi-provision-{process_id}-read-back.bin")),
        }
    }
}

impl Drop for ProvisionTemporaryFiles {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.precheck);
        let _ = fs::remove_file(&self.identity);
        let _ = fs::remove_file(&self.read_back);
    }
}
