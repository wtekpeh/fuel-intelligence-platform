#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceOperationalState {
    Moving,
    Idle,
    Parked,
    Offline,
    Unknown,
}

impl DeviceOperationalState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceOperationalState::Moving => "MOVING",
            DeviceOperationalState::Idle => "IDLE",
            DeviceOperationalState::Parked => "PARKED",
            DeviceOperationalState::Offline => "OFFLINE",
            DeviceOperationalState::Unknown => "UNKNOWN",
        }
    }
}

pub fn classify_device_state(
    device_status: Option<&str>,
    vibration_level: Option<f64>,
    motion_detected: Option<bool>,

    previous_latitude: Option<f64>,
    previous_longitude: Option<f64>,

    current_latitude: Option<f64>,
    current_longitude: Option<f64>,
) -> DeviceOperationalState {
    if matches!(device_status, Some("OFFLINE")) {
        return DeviceOperationalState::Offline;
    }

    let Some(vibration_level) = vibration_level else {
        return DeviceOperationalState::Unknown;
    };

    let motion_detected = motion_detected.unwrap_or(false);

    let gps_moved = match (
        previous_latitude,
        previous_longitude,
        current_latitude,
        current_longitude,
    ) {
        (Some(prev_lat), Some(prev_lon), Some(curr_lat), Some(curr_lon)) => {
            let distance_meters = calculate_distance_meters(prev_lat, prev_lon, curr_lat, curr_lon);

            distance_meters >= 10.0
        }
        _ => false,
    };

    if gps_moved && vibration_level >= 2.0 {
        DeviceOperationalState::Moving
    } else if motion_detected && vibration_level >= 4.0 {
        DeviceOperationalState::Moving
    } else if vibration_level >= 1.0 {
        DeviceOperationalState::Idle
    } else {
        DeviceOperationalState::Parked
    }
}

pub fn calculate_distance_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_meters = 6_371_000.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();

    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_meters * c
}

pub fn calculate_speed_kmh(distance_meters: f64, time_seconds: f64) -> f64 {
    if time_seconds <= 0.0 {
        return 0.0;
    }

    let meters_per_second = distance_meters / time_seconds;

    meters_per_second * 3.6
}

#[test]
fn calculates_speed_kmh() {
    let speed = calculate_speed_kmh(1000.0, 60.0);

    assert!(speed > 0.0);
}

#[test]
fn calculates_distance_between_coordinates() {
    let distance = calculate_distance_meters(5.6037, -0.1870, 5.60385, -0.18688);

    assert!(distance > 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_offline_device() {
        let state = classify_device_state(
            Some("OFFLINE"),
            Some(8.0),
            Some(true),
            None,
            None,
            None,
            None,
        );
        assert_eq!(state, DeviceOperationalState::Offline);
    }

    #[test]
    fn classifies_moving_device() {
        let state = classify_device_state(
            Some("ONLINE"),
            Some(7.0),
            Some(true),
            None,
            None,
            None,
            None,
        );
        assert_eq!(state, DeviceOperationalState::Moving);
    }

    #[test]
    fn classifies_idle_device() {
        let state = classify_device_state(
            Some("ONLINE"),
            Some(2.0),
            Some(false),
            None,
            None,
            None,
            None,
        );
        assert_eq!(state, DeviceOperationalState::Idle);
    }

    #[test]
    fn classifies_parked_device() {
        let state = classify_device_state(
            Some("ONLINE"),
            Some(0.4),
            Some(false),
            None,
            None,
            None,
            None,
        );
        assert_eq!(state, DeviceOperationalState::Parked);
    }

    #[test]
    fn classifies_unknown_when_vibration_missing() {
        let state =
            classify_device_state(Some("ONLINE"), None, Some(false), None, None, None, None);
        assert_eq!(state, DeviceOperationalState::Unknown);
    }
}
