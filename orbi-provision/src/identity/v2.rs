use std::str;

use super::auth::hmac_sha256::{HMAC_SHA256_STORED_TAG_SIZE, calculate_tag, verify_tag};
use super::auth::key::ManufacturingKey;
use super::{
    AUTH_ALGORITHM_HMAC_SHA256, DEVICE_CODE_CAPACITY, DEVICE_CODE_OFFSET, FORMAT_VERSION_V2,
    HMAC_TAG_OFFSET, HMAC_TAG_SIZE, IDENTITY_RECORD_SIZE, MAGIC, PROVISIONED_FLAG, RESERVED_OFFSET,
    RESERVED_SIZE,
};

const AUTHENTICATED_PAYLOAD_SIZE: usize = HMAC_TAG_OFFSET + RESERVED_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedIdentityV2 {
    pub device_code: String,
    pub provisioned: bool,
    pub authentication_tag: [u8; HMAC_SHA256_STORED_TAG_SIZE],
}

pub fn build_identity_record_v2(
    device_code: &str,
    key: &ManufacturingKey,
) -> Result<[u8; IDENTITY_RECORD_SIZE], String> {
    let device_code_bytes = device_code.as_bytes();

    if device_code_bytes.is_empty() {
        return Err(String::from("Device code cannot be empty."));
    }

    if device_code_bytes.len() > DEVICE_CODE_CAPACITY {
        return Err(format!(
            "Device code is too long: {} bytes. Maximum is {} bytes.",
            device_code_bytes.len(),
            DEVICE_CODE_CAPACITY
        ));
    }

    let mut record = [0xFF_u8; IDENTITY_RECORD_SIZE];

    record[0..4].copy_from_slice(&MAGIC);
    record[4] = FORMAT_VERSION_V2;
    record[5] = PROVISIONED_FLAG;
    record[6] = device_code_bytes.len() as u8;
    record[7] = AUTH_ALGORITHM_HMAC_SHA256;

    let device_code_end = DEVICE_CODE_OFFSET + device_code_bytes.len();

    record[DEVICE_CODE_OFFSET..device_code_end].copy_from_slice(device_code_bytes);

    let authenticated_payload = build_authenticated_payload(&record);

    let authentication_tag = calculate_tag(&authenticated_payload, key)?;

    record[HMAC_TAG_OFFSET..HMAC_TAG_OFFSET + HMAC_TAG_SIZE].copy_from_slice(&authentication_tag);

    Ok(record)
}

pub fn decode_identity_record_v2(
    record: &[u8],
    key: &ManufacturingKey,
) -> Result<DecodedIdentityV2, String> {
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

    if record[4] != FORMAT_VERSION_V2 {
        return Err(format!(
            "V2 decoder received identity format version {}.",
            record[4]
        ));
    }

    let provisioned = record[5] & PROVISIONED_FLAG != 0;

    if !provisioned {
        return Err(String::from(
            "Identity record is not marked as provisioned.",
        ));
    }

    if record[7] != AUTH_ALGORITHM_HMAC_SHA256 {
        return Err(format!(
            "Unsupported V2 authentication algorithm: {}.",
            record[7]
        ));
    }

    let device_code_length = usize::from(record[6]);

    if device_code_length == 0 || device_code_length > DEVICE_CODE_CAPACITY {
        return Err(format!(
            "Invalid stored device-code length: {device_code_length}."
        ));
    }

    let device_code_end = DEVICE_CODE_OFFSET + device_code_length;

    if device_code_end > HMAC_TAG_OFFSET {
        return Err(String::from(
            "Stored device code exceeds its allocated record area.",
        ));
    }

    let device_code = str::from_utf8(&record[DEVICE_CODE_OFFSET..device_code_end])
        .map_err(|_| String::from("Stored device code is not valid UTF-8."))?
        .to_owned();

    let mut stored_tag = [0_u8; HMAC_SHA256_STORED_TAG_SIZE];

    stored_tag.copy_from_slice(&record[HMAC_TAG_OFFSET..HMAC_TAG_OFFSET + HMAC_TAG_SIZE]);

    let authenticated_payload = build_authenticated_payload(record);

    verify_tag(&authenticated_payload, &stored_tag, key)?;

    Ok(DecodedIdentityV2 {
        device_code,
        provisioned,
        authentication_tag: stored_tag,
    })
}

fn build_authenticated_payload(record: &[u8]) -> [u8; AUTHENTICATED_PAYLOAD_SIZE] {
    let mut payload = [0_u8; AUTHENTICATED_PAYLOAD_SIZE];

    payload[..HMAC_TAG_OFFSET].copy_from_slice(&record[..HMAC_TAG_OFFSET]);

    payload[HMAC_TAG_OFFSET..]
        .copy_from_slice(&record[RESERVED_OFFSET..RESERVED_OFFSET + RESERVED_SIZE]);

    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> ManufacturingKey {
        ManufacturingKey::from_bytes(vec![0x42; 32]).expect("test key should be valid")
    }

    #[test]
    fn v2_record_can_be_built_and_decoded() {
        let key = test_key();

        let record = build_identity_record_v2("ORBI-A100-000004", &key)
            .expect("V2 record should be generated");

        let decoded = decode_identity_record_v2(&record, &key).expect("V2 record should verify");

        assert_eq!(decoded.device_code, "ORBI-A100-000004");

        assert!(decoded.provisioned);
    }

    #[test]
    fn modified_device_code_is_rejected() {
        let key = test_key();

        let mut record = build_identity_record_v2("ORBI-A100-000004", &key)
            .expect("V2 record should be generated");

        record[DEVICE_CODE_OFFSET] = b'X';

        assert!(decode_identity_record_v2(&record, &key).is_err());
    }

    #[test]
    fn incorrect_key_is_rejected() {
        let correct_key = test_key();

        let incorrect_key = ManufacturingKey::from_bytes(vec![0x43; 32])
            .expect("incorrect test key should be valid");

        let record = build_identity_record_v2("ORBI-A100-000004", &correct_key)
            .expect("V2 record should be generated");

        assert!(decode_identity_record_v2(&record, &incorrect_key).is_err());
    }

    #[test]
    fn modified_header_is_rejected() {
        let key = test_key();

        let mut record = build_identity_record_v2("ORBI-A100-000004", &key)
            .expect("V2 record should be generated");

        record[5] ^= 0x02;

        assert!(decode_identity_record_v2(&record, &key).is_err());
    }

    #[test]
    fn modified_reserved_byte_is_rejected() {
        let key = test_key();

        let mut record = build_identity_record_v2("ORBI-A100-000004", &key)
            .expect("V2 record should be generated");

        record[RESERVED_OFFSET] = 0x00;

        assert!(decode_identity_record_v2(&record, &key).is_err());
    }
}
