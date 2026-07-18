use super::DEVICE_CODE_CAPACITY;

pub fn validate_device_code(device_code: &str) -> Result<(), String> {
    if device_code.is_empty() {
        return Err(String::from("Device code cannot be empty."));
    }

    if device_code.len() > DEVICE_CODE_CAPACITY {
        return Err(format!(
            "Device code cannot exceed {DEVICE_CODE_CAPACITY} characters."
        ));
    }

    if !device_code.is_ascii() {
        return Err(String::from(
            "Device code must contain ASCII characters only.",
        ));
    }

    let valid_characters = device_code
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-');

    if !valid_characters {
        return Err(String::from(
            "Device code may contain only uppercase letters, numbers, and hyphens.",
        ));
    }

    Ok(())
}
