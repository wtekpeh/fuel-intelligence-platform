use super::auth::key::ManufacturingKey;
use super::v1::{build_identity_record_v1, decode_identity_record_v1};
use super::v2::decode_identity_record_v2;
use super::{FORMAT_VERSION_V1, FORMAT_VERSION_V2, IDENTITY_RECORD_SIZE, MAGIC};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityAuthentication {
    /// Version 1 integrity verification using FNV-1a.
    Fnv1a { checksum: u32 },

    /// Version 2 authentication using truncated HMAC-SHA-256.
    HmacSha256 { tag: [u8; 16] },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedIdentity {
    pub device_code: String,
    pub format_version: u8,
    pub provisioned: bool,
    pub authentication: IdentityAuthentication,
}

pub fn build_identity_record(device_code: &str) -> [u8; IDENTITY_RECORD_SIZE] {
    build_identity_record_v1(device_code)
}

/// Decode an identity that does not require a manufacturing key.
///
/// Version 1 records can be decoded here. Version 2 records require
/// `decode_identity_record_with_key`.
pub fn decode_identity_record(record: &[u8]) -> Result<DecodedIdentity, String> {
    validate_record_envelope(record)?;

    match record[4] {
        FORMAT_VERSION_V1 => decode_v1(record),

        FORMAT_VERSION_V2 => Err(String::from(
            "Identity format V2 requires a manufacturing key for authentication.",
        )),

        unsupported_version => Err(format!(
            "Unsupported identity format version: {unsupported_version}."
        )),
    }
}

/// Decode either a Version 1 or Version 2 identity.
///
/// The manufacturing key is used only when the record is Version 2.
pub fn decode_identity_record_with_key(
    record: &[u8],
    key: &ManufacturingKey,
) -> Result<DecodedIdentity, String> {
    validate_record_envelope(record)?;

    match record[4] {
        FORMAT_VERSION_V1 => decode_v1(record),

        FORMAT_VERSION_V2 => {
            let decoded = decode_identity_record_v2(record, key)?;

            Ok(DecodedIdentity {
                device_code: decoded.device_code,
                format_version: FORMAT_VERSION_V2,
                provisioned: decoded.provisioned,
                authentication: IdentityAuthentication::HmacSha256 {
                    tag: decoded.authentication_tag,
                },
            })
        }

        unsupported_version => Err(format!(
            "Unsupported identity format version: {unsupported_version}."
        )),
    }
}

fn decode_v1(record: &[u8]) -> Result<DecodedIdentity, String> {
    let decoded = decode_identity_record_v1(record)?;

    Ok(DecodedIdentity {
        device_code: decoded.device_code,
        format_version: FORMAT_VERSION_V1,
        provisioned: decoded.provisioned,
        authentication: IdentityAuthentication::Fnv1a {
            checksum: decoded.checksum,
        },
    })
}

fn validate_record_envelope(record: &[u8]) -> Result<(), String> {
    if record.len() != IDENTITY_RECORD_SIZE {
        return Err(format!(
            "Identity record has invalid size: {} bytes. Expected {} bytes.",
            record.len(),
            IDENTITY_RECORD_SIZE
        ));
    }

    if record.iter().all(|byte| *byte == 0xFF) {
        return Err(String::from("Identity record is blank or erased."));
    }

    if record[0..4] != MAGIC {
        return Err(String::from("Identity record has an invalid magic value."));
    }

    Ok(())
}
