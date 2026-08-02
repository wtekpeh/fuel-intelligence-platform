use crate::domain::telemetry::motion_buffer::MotionEvidence;

/// Minimum GPS displacement required between consecutive telemetry
/// readings before the asset is considered to have travelled.
///
/// This initial threshold helps suppress small GNSS position fluctuations.
/// It should later be calibrated using real vehicle telemetry.
const GPS_MOVEMENT_THRESHOLD_METERS: f64 = 10.0;

/// Rolling vibration level that indicates stationary operational activity.
///
/// For a vehicle, this can represent engine vibration while the vehicle
/// itself is not travelling.
const IDLE_VIBRATION_THRESHOLD: f64 = 0.7;

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

    pub fn from_str(value: &str) -> Self {
        match value {
            "MOVING" => DeviceOperationalState::Moving,
            "IDLE" => DeviceOperationalState::Idle,
            "PARKED" => DeviceOperationalState::Parked,
            "OFFLINE" => DeviceOperationalState::Offline,
            "UNKNOWN" => DeviceOperationalState::Unknown,
            _ => DeviceOperationalState::Unknown,
        }
    }
}

/// Classifies a device's operational state using GPS displacement and
/// rolling IMU evidence.
///
/// Classification priority:
///
/// 1. OFFLINE remains the highest-priority state.
/// 2. Meaningful GPS displacement confirms that the asset is MOVING.
/// 3. Sustained rolling IMU motion also confirms MOVING.
/// 4. Non-sustained vibration indicates IDLE.
/// 5. Low vibration with no GPS movement indicates PARKED.
///
/// GPS is allowed to confirm vehicle travel independently because a vehicle
/// can move smoothly without producing strong vibration measurements.
pub fn classify_device_state_from_motion(
    device_status: Option<&str>,
    motion_evidence: Option<&MotionEvidence>,

    previous_latitude: Option<f64>,
    previous_longitude: Option<f64>,

    current_latitude: Option<f64>,
    current_longitude: Option<f64>,
) -> DeviceOperationalState {
    if matches!(device_status, Some("OFFLINE")) {
        return DeviceOperationalState::Offline;
    }

    let Some(motion_evidence) = motion_evidence else {
        return DeviceOperationalState::Unknown;
    };

    let gps_moved = has_meaningful_gps_displacement(
        previous_latitude,
        previous_longitude,
        current_latitude,
        current_longitude,
    );

    if gps_moved {
        DeviceOperationalState::Moving
    } else if motion_evidence.sustained_motion {
        DeviceOperationalState::Moving
    } else if motion_evidence.average_vibration_score >= IDLE_VIBRATION_THRESHOLD {
        DeviceOperationalState::Idle
    } else {
        DeviceOperationalState::Parked
    }
}

/// Legacy single-reading classifier.
///
/// This remains temporarily available while the rolling motion classifier
/// is compared against the previous implementation.
///
/// Its sensor-fusion priority now matches the authoritative rolling
/// classifier so GPS-confirmed travel is not rejected by weak vibration.
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

    let gps_moved = has_meaningful_gps_displacement(
        previous_latitude,
        previous_longitude,
        current_latitude,
        current_longitude,
    );

    if gps_moved {
        DeviceOperationalState::Moving
    } else if motion_detected && vibration_level >= 4.0 {
        DeviceOperationalState::Moving
    } else if vibration_level >= IDLE_VIBRATION_THRESHOLD {
        DeviceOperationalState::Idle
    } else {
        DeviceOperationalState::Parked
    }
}

/// Determines whether two GPS positions are separated by enough distance
/// to represent meaningful asset movement.
pub fn has_meaningful_gps_displacement(
    previous_latitude: Option<f64>,
    previous_longitude: Option<f64>,
    current_latitude: Option<f64>,
    current_longitude: Option<f64>,
) -> bool {
    match (
        previous_latitude,
        previous_longitude,
        current_latitude,
        current_longitude,
    ) {
        (Some(prev_lat), Some(prev_lon), Some(curr_lat), Some(curr_lon)) => {
            let distance_meters = calculate_distance_meters(prev_lat, prev_lon, curr_lat, curr_lon);

            distance_meters >= GPS_MOVEMENT_THRESHOLD_METERS
        }
        _ => false,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn moving_motion_evidence() -> MotionEvidence {
        MotionEvidence {
            average_vibration_score: 5.0,
            average_gravity_deviation_g: 0.0,
            average_rotation_magnitude_dps: 0.0,
            motion_ratio: 0.8,
            average_confidence: 0.9,
            sustained_motion: true,
            sample_count: 5,
        }
    }

    fn idle_motion_evidence() -> MotionEvidence {
        MotionEvidence {
            average_vibration_score: 2.0,
            average_gravity_deviation_g: 0.0,
            average_rotation_magnitude_dps: 0.0,
            motion_ratio: 0.2,
            average_confidence: 0.4,
            sustained_motion: false,
            sample_count: 5,
        }
    }

    fn parked_motion_evidence() -> MotionEvidence {
        MotionEvidence {
            average_vibration_score: 0.3,
            average_gravity_deviation_g: 0.0,
            average_rotation_magnitude_dps: 0.0,
            motion_ratio: 0.0,
            average_confidence: 0.0,
            sustained_motion: false,
            sample_count: 5,
        }
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

    #[test]
    fn motion_classifier_preserves_offline_state() {
        let evidence = moving_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("OFFLINE"),
            Some(&evidence),
            None,
            None,
            None,
            None,
        );

        assert_eq!(state, DeviceOperationalState::Offline);
    }

    #[test]
    fn motion_classifier_returns_unknown_without_motion_evidence() {
        let state = classify_device_state_from_motion(Some("ONLINE"), None, None, None, None, None);

        assert_eq!(state, DeviceOperationalState::Unknown);
    }

    #[test]
    fn motion_classifier_classifies_gps_displacement_as_moving() {
        let evidence = parked_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("ONLINE"),
            Some(&evidence),
            Some(51.8773689),
            Some(-0.4309177),
            Some(51.8767586),
            Some(-0.4311118),
        );

        assert_eq!(state, DeviceOperationalState::Moving);
    }

    #[test]
    fn motion_classifier_classifies_sustained_motion_as_moving() {
        let evidence = moving_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("ONLINE"),
            Some(&evidence),
            None,
            None,
            None,
            None,
        );

        assert_eq!(state, DeviceOperationalState::Moving);
    }

    #[test]
    fn motion_classifier_classifies_non_sustained_vibration_as_idle() {
        let evidence = idle_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("ONLINE"),
            Some(&evidence),
            None,
            None,
            None,
            None,
        );

        assert_eq!(state, DeviceOperationalState::Idle);
    }

    #[test]
    fn motion_classifier_classifies_low_vibration_as_parked() {
        let evidence = parked_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("ONLINE"),
            Some(&evidence),
            None,
            None,
            None,
            None,
        );

        assert_eq!(state, DeviceOperationalState::Parked);
    }

    #[test]
    fn motion_classifier_ignores_small_gps_displacement() {
        let evidence = parked_motion_evidence();

        let state = classify_device_state_from_motion(
            Some("ONLINE"),
            Some(&evidence),
            Some(51.8776000),
            Some(-0.4292000),
            Some(51.8776050),
            Some(-0.4292050),
        );

        assert_eq!(state, DeviceOperationalState::Parked);
    }

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
    fn legacy_classifier_classifies_gps_displacement_as_moving() {
        let state = classify_device_state(
            Some("ONLINE"),
            Some(0.2),
            Some(false),
            Some(51.8773689),
            Some(-0.4309177),
            Some(51.8767586),
            Some(-0.4311118),
        );

        assert_eq!(state, DeviceOperationalState::Moving);
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
