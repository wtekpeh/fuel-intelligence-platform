use std::str;

use super::checksum::calculate_checksum;
use super::{
    CHECKSUM_OFFSET, DEVICE_CODE_CAPACITY, DEVICE_CODE_OFFSET, FORMAT_VERSION_V1,
    IDENTITY_RECORD_SIZE, MAGIC, PROVISIONED_FLAG,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedIdentityV1 {
    pub device_code: String,
    pub provisioned: bool,
    pub checksum: u32,
}

pub fn build_identity_record_v1(device_code: &str) -> [u8; IDENTITY_RECORD_SIZE] {
    let device_code_bytes = device_code.as_bytes();

    let mut record = [0xFF_u8; IDENTITY_RECORD_SIZE];

    record[0..4].copy_from_slice(&MAGIC);
    record[4] = FORMAT_VERSION_V1;
    record[5] = PROVISIONED_FLAG;
    record[6] = device_code_bytes.len() as u8;
    record[7] = 0xFF;

    let device_code_end = DEVICE_CODE_OFFSET + device_code_bytes.len();

    record[DEVICE_CODE_OFFSET..device_code_end].copy_from_slice(device_code_bytes);

    let checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    record[CHECKSUM_OFFSET..CHECKSUM_OFFSET + 4].copy_from_slice(&checksum.to_le_bytes());

    record
}

pub fn decode_identity_record_v1(record: &[u8]) -> Result<DecodedIdentityV1, String> {
    if record.len() != IDENTITY_RECORD_SIZE {
        return Err(format!(
            "Identity record has invalid size: {} bytes. Expected {} bytes.",
            record.len(),
            IDENTITY_RECORD_SIZE
        ));
    }

    if record[0..4] != MAGIC {
        return Err(String::from("Identity record has an invalid magic value."));
    }

    if record[4] != FORMAT_VERSION_V1 {
        return Err(format!(
            "V1 decoder received identity format version {}.",
            record[4]
        ));
    }

    let provisioned = record[5] & PROVISIONED_FLAG != 0;

    if !provisioned {
        return Err(String::from(
            "Identity record is not marked as provisioned.",
        ));
    }

    let device_code_length = usize::from(record[6]);

    if device_code_length == 0 || device_code_length > DEVICE_CODE_CAPACITY {
        return Err(format!(
            "Invalid stored device-code length: {device_code_length}."
        ));
    }

    let device_code_end = DEVICE_CODE_OFFSET + device_code_length;

    if device_code_end > CHECKSUM_OFFSET {
        return Err(String::from(
            "Stored device code exceeds its allocated record area.",
        ));
    }

    let device_code = str::from_utf8(&record[DEVICE_CODE_OFFSET..device_code_end])
        .map_err(|_| String::from("Stored device code is not valid UTF-8."))?
        .to_owned();

    let stored_checksum = u32::from_le_bytes([
        record[CHECKSUM_OFFSET],
        record[CHECKSUM_OFFSET + 1],
        record[CHECKSUM_OFFSET + 2],
        record[CHECKSUM_OFFSET + 3],
    ]);

    let calculated_checksum = calculate_checksum(&record[..CHECKSUM_OFFSET]);

    if stored_checksum != calculated_checksum {
        return Err(format!(
            "Identity checksum verification failed. \
             Stored: 0x{stored_checksum:08X}, \
             calculated: 0x{calculated_checksum:08X}."
        ));
    }

    Ok(DecodedIdentityV1 {
        device_code,
        provisioned,
        checksum: stored_checksum,
    })
}
