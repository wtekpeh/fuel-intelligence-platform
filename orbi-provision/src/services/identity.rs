use crate::identity::IDENTITY_RECORD_SIZE;
use crate::identity::auth::key::ManufacturingKey;
use crate::identity::record::{
    DecodedIdentity, build_identity_record, decode_identity_record, decode_identity_record_with_key,
};
use crate::identity::v2::build_identity_record_v2;

pub fn generate_v1_identity(device_code: &str) -> [u8; IDENTITY_RECORD_SIZE] {
    build_identity_record(device_code)
}

pub fn generate_v2_identity(
    device_code: &str,
    key: &ManufacturingKey,
) -> Result<[u8; IDENTITY_RECORD_SIZE], String> {
    build_identity_record_v2(device_code, key)
}

pub fn verify_v1_identity(record: &[u8]) -> Result<DecodedIdentity, String> {
    decode_identity_record(record)
}

pub fn verify_v2_identity(
    record: &[u8],
    key: &ManufacturingKey,
) -> Result<DecodedIdentity, String> {
    decode_identity_record_with_key(record, key)
}
