pub fn convert_nmea_latitude(value: &str, hemisphere: &str) -> Option<f64> {
    if value.len() < 4 {
        return None;
    }

    let degrees: f64 = value[0..2].parse().ok()?;
    let minutes: f64 = value[2..].parse().ok()?;

    let mut decimal = degrees + (minutes / 60.0);

    if hemisphere == "S" {
        decimal *= -1.0;
    }

    Some(decimal)
}

pub fn convert_nmea_longitude(value: &str, hemisphere: &str) -> Option<f64> {
    if value.len() < 5 {
        return None;
    }

    let degrees: f64 = value[0..3].parse().ok()?;
    let minutes: f64 = value[3..].parse().ok()?;

    let mut decimal = degrees + (minutes / 60.0);

    if hemisphere == "W" {
        decimal *= -1.0;
    }

    Some(decimal)
}

pub struct GpsInfo {
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64,
    pub timestamp: heapless::String<32>,
}

pub fn parse_cgpsinfo_response(buffer: &[u8]) -> Option<GpsInfo> {
    let response = core::str::from_utf8(buffer).ok()?;

    let gps_line = response.lines().find(|line| line.contains("+CGPSINFO:"))?;

    let data = gps_line.split("+CGPSINFO:").nth(1)?.trim();

    let mut parts = data.split(',');

    let lat_value = parts.next()?.trim();
    let lat_hemisphere = parts.next()?.trim();

    let lon_value = parts.next()?.trim();
    let lon_hemisphere = parts.next()?.trim();

    let date = parts.next()?.trim();
    let time = parts.next()?.trim();

    let timestamp = build_iso_timestamp(date, time)?;
    let _altitude = parts.next()?.trim();

    let speed_text = parts.next()?.trim();
    let speed = speed_text.parse::<f64>().unwrap_or(0.0);

    let latitude = convert_nmea_latitude(lat_value, lat_hemisphere)?;
    let longitude = convert_nmea_longitude(lon_value, lon_hemisphere)?;

    Some(GpsInfo {
        latitude,
        longitude,
        speed,
        timestamp,
    })
}

pub fn build_iso_timestamp(date: &str, time: &str) -> Option<heapless::String<32>> {
    if date.len() != 6 || time.len() < 6 {
        return None;
    }

    let day = &date[0..2];
    let month = &date[2..4];
    let year_suffix = &date[4..6];

    let hour = &time[0..2];
    let minute = &time[2..4];
    let second = &time[4..6];

    let mut timestamp = heapless::String::<32>::new();

    let _ = core::fmt::write(
        &mut timestamp,
        format_args!(
            "20{}-{}-{}T{}:{}:{}Z",
            year_suffix, month, day, hour, minute, second
        ),
    );

    Some(timestamp)
}
