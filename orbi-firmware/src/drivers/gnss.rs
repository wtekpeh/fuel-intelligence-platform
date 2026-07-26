use esp_hal::delay::Delay;
use esp_println::{print, println};
use heapless::{String, Vec};

use crate::drivers::Modem;

const MAX_GNSS_FIELDS: usize = 20;

pub fn initialize(modem: &mut Modem, delay: &Delay) {
    println!("========================");
    println!("ORBI GNSS INITIALIZATION");
    println!("========================");

    /*
     * Power on the GNSS subsystem.
     */
    modem.send_command_and_print_response(b"AT+CGNSSPWR=1\r\n", "AT+CGNSSPWR=1", delay);

    /*
     * Give the modem enough time to initialise GNSS.
     *
     * The modem may report:
     *
     * +CGNSSPWR: READY!
     */
    delay.delay_millis(3000);

    /*
     * Confirm that GNSS power is enabled.
     */
    modem.send_command_and_print_response(b"AT+CGNSSPWR?\r\n", "AT+CGNSSPWR?", delay);

    delay.delay_millis(1000);

    /*
     * Request an initial multi-constellation GNSS result.
     *
     * A valid fix may not yet be available during startup. This command
     * is useful for confirming that the modem accepts AT+CGNSSINFO.
     */
    modem.send_command_and_print_response(b"AT+CGNSSINFO\r\n", "AT+CGNSSINFO", delay);

    delay.delay_millis(1000);

    println!("GNSS initialization commands completed.");
}

pub fn get_live_fix(modem: &mut Modem, delay: &Delay) -> Option<GpsInfo> {
    /*
     * AT+CGNSSINFO provides:
     *
     * - fix mode
     * - satellite counts
     * - latitude and longitude
     * - date and UTC time
     * - altitude
     * - speed over ground in knots
     * - course over ground in degrees
     * - PDOP, HDOP and VDOP
     */
    let (response_buffer, bytes_read) =
        modem.send_command_and_collect_response(b"AT+CGNSSINFO\r\n", "AT+CGNSSINFO", delay)?;

    println!("========================");
    println!("RAW GNSS RESPONSE");
    println!("========================");

    for byte in &response_buffer[..bytes_read] {
        print!("{}", *byte as char);
    }

    println!();

    let gps_info = parse_cgnssinfo_response(&response_buffer[..bytes_read])?;

    println!("========================");
    println!("PARSED GNSS SOLUTION");
    println!("========================");
    println!("Fix Mode: {}", gps_info.fix_mode);
    println!("Latitude: {}", gps_info.latitude);
    println!("Longitude: {}", gps_info.longitude);
    println!("Altitude: {} metres", gps_info.altitude_metres);
    println!("Speed: {} knots", gps_info.speed);
    println!("Speed: {} km/h", gps_info.speed * 1.852);
    println!("Heading: {} degrees", gps_info.heading);
    println!("PDOP: {}", gps_info.pdop);
    println!("HDOP: {}", gps_info.hdop);
    println!("VDOP: {}", gps_info.vdop);
    println!("Timestamp: {}", gps_info.timestamp);

    Some(gps_info)
}

pub struct GpsInfo {
    pub fix_mode: u8,

    pub latitude: f64,
    pub longitude: f64,
    pub altitude_metres: f64,

    /*
     * Speed over ground reported by AT+CGNSSINFO.
     *
     * Unit: knots.
     *
     * This remains named `speed` temporarily so the existing publisher
     * and reporting scheduler continue to compile without unrelated
     * changes.
     */
    pub speed: f64,

    /*
     * Course over ground.
     *
     * Unit: degrees.
     */
    pub heading: f64,

    pub pdop: f64,
    pub hdop: f64,
    pub vdop: f64,

    pub timestamp: String<32>,
}

pub fn parse_cgnssinfo_response(buffer: &[u8]) -> Option<GpsInfo> {
    let response = core::str::from_utf8(buffer).ok()?;

    /*
     * Find the actual GNSS result line while ignoring:
     *
     * - command echo
     * - OK
     * - unrelated modem output
     */
    let gnss_line = response
        .lines()
        .find(|line| line.trim_start().starts_with("+CGNSSINFO:"))?;

    let data = gnss_line.split_once(':')?.1.trim();

    /*
     * A response without a GNSS fix may look like:
     *
     * +CGNSSINFO:,,,,,,,,,,,,,,,
     */
    if data.is_empty() {
        println!("GNSS response contains no navigation data.");

        return None;
    }

    let mut fields: Vec<&str, MAX_GNSS_FIELDS> = Vec::new();

    for field in data.split(',') {
        fields.push(field.trim()).ok()?;
    }

    if fields.is_empty() {
        println!("GNSS response contains no fields.");

        return None;
    }

    /*
     * The first field remains the GNSS fix mode.
     */
    let fix_mode_text = fields[0];

    if fix_mode_text.is_empty() {
        println!("GNSS fix mode is missing.");

        return None;
    }

    let fix_mode = match fix_mode_text.parse::<u8>() {
        Ok(value) => value,

        Err(_) => {
            println!("Could not parse GNSS fix mode. Value: '{}'", fix_mode_text);

            return None;
        }
    };

    /*
     * According to the modem format:
     *
     * 2 = 2D fix
     * 3 = 3D fix
     */
    if fix_mode != 2 && fix_mode != 3 {
        println!("GNSS does not currently have a valid fix.");
        println!("Reported Fix Mode: {}", fix_mode);

        return None;
    }

    /*
     * Do not calculate the navigation start from the end of the response.
     *
     * A76XX modem firmware variants may append extra satellite fields or
     * leave optional fields empty.
     *
     * Instead, locate the navigation block structurally:
     *
     * latitude
     * N/S
     * longitude
     * E/W
     * date
     * time
     */
    let mut navigation_start: Option<usize> = None;

    if fields.len() >= 6 {
        for index in 1..=(fields.len() - 6) {
            let latitude_value = fields[index];
            let latitude_hemisphere = fields[index + 1];

            let longitude_value = fields[index + 2];
            let longitude_hemisphere = fields[index + 3];

            let date = fields[index + 4];
            let time = fields[index + 5];

            let latitude_is_numeric = latitude_value.parse::<f64>().is_ok();

            let longitude_is_numeric = longitude_value.parse::<f64>().is_ok();

            let latitude_hemisphere_is_valid =
                latitude_hemisphere == "N" || latitude_hemisphere == "S";

            let longitude_hemisphere_is_valid =
                longitude_hemisphere == "E" || longitude_hemisphere == "W";

            let date_looks_valid =
                date.len() == 6 && date.as_bytes().iter().all(|byte| byte.is_ascii_digit());

            let time_looks_valid = time.len() >= 6;

            if latitude_is_numeric
                && longitude_is_numeric
                && latitude_hemisphere_is_valid
                && longitude_hemisphere_is_valid
                && date_looks_valid
                && time_looks_valid
            {
                navigation_start = Some(index);

                break;
            }
        }
    }

    let navigation_start = match navigation_start {
        Some(index) => index,

        None => {
            println!("Could not locate the GNSS navigation field sequence.");

            return None;
        }
    };

    /*
     * The navigation block requires twelve fields beginning at the
     * detected latitude position.
     *
     * Extra fields after VDOP are ignored.
     */
    if navigation_start + 11 >= fields.len() {
        println!(
            "GNSS navigation block is incomplete. Start index: {}, total fields: {}",
            navigation_start,
            fields.len()
        );

        return None;
    }

    let latitude_value = fields[navigation_start];

    let latitude_hemisphere = fields[navigation_start + 1];

    let longitude_value = fields[navigation_start + 2];

    let longitude_hemisphere = fields[navigation_start + 3];

    let date = fields[navigation_start + 4];

    let time = fields[navigation_start + 5];

    let altitude_text = fields[navigation_start + 6];

    let speed_text = fields[navigation_start + 7];

    let heading_text = fields[navigation_start + 8];

    let pdop_text = fields[navigation_start + 9];

    let hdop_text = fields[navigation_start + 10];

    let vdop_text = fields[navigation_start + 11];

    /*
     * Latitude, longitude and timestamp are required because they define
     * the actual position and recording time.
     */
    let latitude = parse_decimal_latitude(latitude_value, latitude_hemisphere)?;

    let longitude = parse_decimal_longitude(longitude_value, longitude_hemisphere)?;

    let timestamp = build_iso_timestamp(date, time)?;

    /*
     * The modem may legitimately leave these fields empty.
     *
     * Examples:
     *
     * - speed may be empty while stationary
     * - heading may be empty before a course has been established
     * - altitude may be unavailable during a 2D fix
     *
     * Empty optional fields therefore use zero, while malformed non-empty
     * fields are still rejected.
     */
    let altitude_metres = parse_optional_f64("altitude", altitude_text, 0.0)?;

    let speed = parse_optional_f64("speed", speed_text, 0.0)?;

    let heading = parse_optional_f64("heading", heading_text, 0.0)?;

    let pdop = parse_optional_f64("PDOP", pdop_text, 0.0)?;

    let hdop = parse_optional_f64("HDOP", hdop_text, 0.0)?;

    let vdop = parse_optional_f64("VDOP", vdop_text, 0.0)?;

    Some(GpsInfo {
        fix_mode,
        latitude,
        longitude,
        altitude_metres,
        speed,
        heading,
        pdop,
        hdop,
        vdop,
        timestamp,
    })
}

fn parse_required_f64(field_name: &str, field_value: &str) -> Option<f64> {
    if field_value.is_empty() {
        println!("GNSS field '{}' is empty.", field_name);

        return None;
    }

    match field_value.parse::<f64>() {
        Ok(value) => Some(value),

        Err(_) => {
            println!(
                "Could not parse GNSS field '{}'. Value: '{}'",
                field_name, field_value
            );

            None
        }
    }
}

fn parse_optional_f64(field_name: &str, field_value: &str, default_value: f64) -> Option<f64> {
    if field_value.is_empty() {
        println!(
            "GNSS optional field '{}' is empty. Using default value: {}",
            field_name, default_value
        );

        return Some(default_value);
    }

    match field_value.parse::<f64>() {
        Ok(value) => Some(value),

        Err(_) => {
            println!(
                "Could not parse GNSS field '{}'. Value: '{}'",
                field_name, field_value
            );

            None
        }
    }
}

fn parse_decimal_latitude(value: &str, hemisphere: &str) -> Option<f64> {
    let parsed_value = parse_required_f64("latitude", value)?;

    if !(0.0..=90.0).contains(&parsed_value) {
        println!(
            "GNSS latitude is outside the valid range. Value: {}",
            parsed_value
        );

        return None;
    }

    match hemisphere {
        "N" => Some(parsed_value),

        "S" => Some(-parsed_value),

        _ => {
            println!("Invalid latitude hemisphere: '{}'", hemisphere);

            None
        }
    }
}

fn parse_decimal_longitude(value: &str, hemisphere: &str) -> Option<f64> {
    let parsed_value = parse_required_f64("longitude", value)?;

    if !(0.0..=180.0).contains(&parsed_value) {
        println!(
            "GNSS longitude is outside the valid range. Value: {}",
            parsed_value
        );

        return None;
    }

    match hemisphere {
        "E" => Some(parsed_value),

        "W" => Some(-parsed_value),

        _ => {
            println!("Invalid longitude hemisphere: '{}'", hemisphere);

            None
        }
    }
}

pub fn convert_nmea_latitude(value: &str, hemisphere: &str) -> Option<f64> {
    if value.len() < 4 {
        return None;
    }

    let degrees = value.get(0..2)?.parse::<f64>().ok()?;

    let minutes = value.get(2..)?.parse::<f64>().ok()?;

    let mut decimal = degrees + (minutes / 60.0);

    match hemisphere {
        "N" => {}

        "S" => {
            decimal *= -1.0;
        }

        _ => {
            println!("Invalid latitude hemisphere: '{}'", hemisphere);

            return None;
        }
    }

    Some(decimal)
}

pub fn convert_nmea_longitude(value: &str, hemisphere: &str) -> Option<f64> {
    if value.len() < 5 {
        return None;
    }

    let degrees = value.get(0..3)?.parse::<f64>().ok()?;

    let minutes = value.get(3..)?.parse::<f64>().ok()?;

    let mut decimal = degrees + (minutes / 60.0);

    match hemisphere {
        "E" => {}

        "W" => {
            decimal *= -1.0;
        }

        _ => {
            println!("Invalid longitude hemisphere: '{}'", hemisphere);

            return None;
        }
    }

    Some(decimal)
}

pub fn build_iso_timestamp(date: &str, time: &str) -> Option<String<32>> {
    if date.len() != 6 || time.len() < 6 {
        println!("Invalid GNSS date/time. Date: '{}', Time: '{}'", date, time);

        return None;
    }

    let day = date.get(0..2)?;

    let month = date.get(2..4)?;

    let year_suffix = date.get(4..6)?;

    let hour = time.get(0..2)?;

    let minute = time.get(2..4)?;

    let second = time.get(4..6)?;

    let mut timestamp = String::<32>::new();

    core::fmt::write(
        &mut timestamp,
        format_args!(
            "20{}-{}-{}T{}:{}:{}Z",
            year_suffix, month, day, hour, minute, second
        ),
    )
    .ok()?;

    Some(timestamp)
}
