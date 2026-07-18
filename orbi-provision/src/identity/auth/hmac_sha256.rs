use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::key::ManufacturingKey;

pub const HMAC_SHA256_STORED_TAG_SIZE: usize = 16;

type HmacSha256 = Hmac<Sha256>;

pub fn calculate_tag(
    payload: &[u8],
    key: &ManufacturingKey,
) -> Result<[u8; HMAC_SHA256_STORED_TAG_SIZE], String> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|_| String::from("Could not initialise HMAC-SHA-256."))?;

    mac.update(payload);

    let full_tag = mac.finalize().into_bytes();

    let mut stored_tag = [0_u8; HMAC_SHA256_STORED_TAG_SIZE];

    stored_tag.copy_from_slice(&full_tag[..HMAC_SHA256_STORED_TAG_SIZE]);

    Ok(stored_tag)
}

pub fn verify_tag(
    payload: &[u8],
    stored_tag: &[u8; HMAC_SHA256_STORED_TAG_SIZE],
    key: &ManufacturingKey,
) -> Result<(), String> {
    let calculated_tag = calculate_tag(payload, key)?;

    if !constant_time_equals(&calculated_tag, stored_tag) {
        return Err(String::from(
            "HMAC-SHA-256 authentication verification failed.",
        ));
    }

    Ok(())
}

fn constant_time_equals(
    left: &[u8; HMAC_SHA256_STORED_TAG_SIZE],
    right: &[u8; HMAC_SHA256_STORED_TAG_SIZE],
) -> bool {
    let mut difference = 0_u8;

    for index in 0..HMAC_SHA256_STORED_TAG_SIZE {
        difference |= left[index] ^ right[index];
    }

    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::auth::key::ManufacturingKey;

    #[test]
    fn generated_tag_can_be_verified() {
        let key = ManufacturingKey::from_bytes(vec![0x42; 32]).expect("test key should be valid");

        let payload = b"ORBI-A100-000004";

        let tag = calculate_tag(payload, &key).expect("tag generation should succeed");

        assert!(verify_tag(payload, &tag, &key).is_ok());
    }

    #[test]
    fn modified_payload_is_rejected() {
        let key = ManufacturingKey::from_bytes(vec![0x42; 32]).expect("test key should be valid");

        let original_payload = b"ORBI-A100-000004";
        let modified_payload = b"ORBI-A100-000005";

        let tag = calculate_tag(original_payload, &key).expect("tag generation should succeed");

        assert!(verify_tag(modified_payload, &tag, &key).is_err());
    }

    #[test]
    fn incorrect_key_is_rejected() {
        let correct_key =
            ManufacturingKey::from_bytes(vec![0x42; 32]).expect("test key should be valid");

        let incorrect_key =
            ManufacturingKey::from_bytes(vec![0x43; 32]).expect("test key should be valid");

        let payload = b"ORBI-A100-000004";

        let tag = calculate_tag(payload, &correct_key).expect("tag generation should succeed");

        assert!(verify_tag(payload, &tag, &incorrect_key).is_err());
    }
}
