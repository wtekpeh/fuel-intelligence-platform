pub mod auth;
pub mod checksum;
pub mod record;
pub mod v1;
pub mod v2;
pub mod validation;

pub const IDENTITY_RECORD_SIZE: usize = 64;
pub const DEVICE_CODE_CAPACITY: usize = 32;

pub const MAGIC: [u8; 4] = *b"ORBI";

pub const FORMAT_VERSION_V1: u8 = 1;
pub const FORMAT_VERSION_V2: u8 = 2;

pub const PROVISIONED_FLAG: u8 = 0x01;

pub const AUTH_ALGORITHM_HMAC_SHA256: u8 = 2;

pub const DEVICE_CODE_OFFSET: usize = 8;

pub const CHECKSUM_OFFSET: usize = 40;

pub const HMAC_TAG_OFFSET: usize = 40;
pub const HMAC_TAG_SIZE: usize = 16;
pub const RESERVED_OFFSET: usize = 56;
pub const RESERVED_SIZE: usize = 8;

pub const IDENTITY_FLASH_ADDRESS: u32 = 0x001F_0000;
