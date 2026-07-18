use std::fs;
use std::path::Path;

pub const MINIMUM_HMAC_KEY_SIZE: usize = 32;

#[derive(Debug, Clone)]
pub struct ManufacturingKey {
    bytes: Vec<u8>,
}

impl ManufacturingKey {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();

        let bytes = fs::read(path).map_err(|error| {
            format!(
                "Could not read manufacturing key file '{}': {}",
                path.display(),
                error
            )
        })?;

        Self::from_bytes(bytes)
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < MINIMUM_HMAC_KEY_SIZE {
            return Err(format!(
                "Manufacturing key is too short: {} bytes. \
                 At least {} bytes are required.",
                bytes.len(),
                MINIMUM_HMAC_KEY_SIZE
            ));
        }

        Ok(Self { bytes })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn length(&self) -> usize {
        self.bytes.len()
    }
}
