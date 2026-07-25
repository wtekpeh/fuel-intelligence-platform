use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::telemetry::imu_interpreter::ImuInterpretation;
use crate::domain::telemetry::motion_buffer::MotionEvidence;
use crate::models::FuelReading;
use crate::repository::{
    self, NewDeviceStateEvent, NewOperationalIntelligenceEvent, NewOperationalTransitionEvent,
    StoredTelemetryPosition, get_latest_device_state,
};
use crate::services::device_state::{
    DeviceOperationalState, calculate_distance_meters, calculate_speed_kmh, classify_device_state,
    classify_device_state_from_motion,
};
use crate::services::operational_behaviour::determine_operational_transition;
use crate::services::operational_intelligence::interpret_operational_transition;

/// Processes shared ORBI motion and operational intelligence.
///
/// This service is available to every ORBI vehicle product because every
/// product includes GPS and vibration capabilities.
///
/// It is responsible for:
///
/// - calculating movement distance and speed,
/// - classifying the current device operational state,
/// - detecting operational state transitions,
/// - generating operational intelligence events,
/// - persisting device-state history.
///
/// Fuel persistence and fuel-event detection do not belong in this service.
pub async fn process_motion_intelligence(
    db_pool: &PgPool,
    device_id: Uuid,
    vibration_sensor_id: Uuid,
    reading: &FuelReading,
    previous_position: Option<&StoredTelemetryPosition>,
    imu_interpretation: &ImuInterpretation,
    motion_evidence: Option<&MotionEvidence>,
) -> Result<()> {
    let (previous_latitude, previous_longitude, distance_meters, speed_kmh) =
        calculate_movement(reading, previous_position);

    println!(
        "[MOVEMENT DEBUG] device_id={}, previous=({:?}, {:?}), current=({}, {}), \
        distance_meters={:?}, speed_kmh={:?}, previous_position_present={}",
        device_id,
        previous_latitude,
        previous_longitude,
        reading.latitude,
        reading.longitude,
        distance_meters,
        speed_kmh,
        previous_position.is_some(),
    );

    // Keep the original single-reading classifier temporarily so its result
    // can be compared with the rolling motion-pipeline classifier.
    //
    // The rolling motion classifier remains the authoritative classifier.
    let legacy_device_state = classify_device_state(
        Some("ONLINE"),
        Some(imu_interpretation.vibration_score),
        Some(imu_interpretation.motion_detected),
        previous_latitude,
        previous_longitude,
        Some(reading.latitude),
        Some(reading.longitude),
    );

    let motion_device_state = classify_device_state_from_motion(
        Some("ONLINE"),
        motion_evidence,
        previous_latitude,
        previous_longitude,
        Some(reading.latitude),
        Some(reading.longitude),
    );

    println!(
        "[CLASSIFIER DEBUG] device_id={}, state={}, legacy_state={}, \
        distance_meters={:?}, speed_kmh={:?}, vibration_score={:.4}, \
        imu_motion_detected={}, motion_evidence={:?}",
        device_id,
        motion_device_state.as_str(),
        legacy_device_state.as_str(),
        distance_meters,
        speed_kmh,
        imu_interpretation.vibration_score,
        imu_interpretation.motion_detected,
        motion_evidence,
    );

    let previous_state = get_latest_device_state(db_pool, device_id)
        .await?
        .map(|state| DeviceOperationalState::from_str(&state));

    let operational_transition =
        determine_operational_transition(previous_state.as_ref(), &motion_device_state);

    let previous_state = get_latest_device_state(db_pool, device_id)
        .await?
        .map(|state| DeviceOperationalState::from_str(&state));

    let state_changed = previous_state.as_ref() != Some(&motion_device_state);

    let operational_transition =
        determine_operational_transition(previous_state.as_ref(), &motion_device_state);

    if operational_transition.occurred()
        && let Some(previous_state) = previous_state.as_ref()
    {
        let transition_event = NewOperationalTransitionEvent {
            device_id,

            previous_state: previous_state.as_str().to_string(),
            current_state: motion_device_state.as_str().to_string(),
            transition: operational_transition.as_str().to_string(),

            latitude: Some(reading.latitude),
            longitude: Some(reading.longitude),

            recorded_at: reading.timestamp,

            source: "MOTION_PIPELINE".to_string(),
        };

        let transition_event_id =
            repository::create_operational_transition_event(db_pool, &transition_event).await?;

        let intelligence_event = interpret_operational_transition(&operational_transition);

        if intelligence_event.occurred() {
            repository::create_operational_intelligence_event(
                db_pool,
                &NewOperationalIntelligenceEvent {
                    device_id,

                    operational_transition_event_id: Some(transition_event_id),

                    event_type: intelligence_event.as_str().to_string(),

                    previous_state: Some(previous_state.as_str().to_string()),
                    current_state: Some(motion_device_state.as_str().to_string()),

                    latitude: Some(reading.latitude),
                    longitude: Some(reading.longitude),

                    recorded_at: reading.timestamp,

                    source: "OPERATIONAL_INTELLIGENCE".to_string(),
                },
            )
            .await?;
        }

        println!(
            "[OPERATIONAL INTELLIGENCE] device_id={}, transition={}, event={}",
            device_id,
            operational_transition.as_str(),
            intelligence_event.as_str(),
        );
    }

    if legacy_device_state != motion_device_state {
        println!(
            "[DEVICE STATE COMPARISON] device_id={}, legacy={}, motion={}",
            device_id,
            legacy_device_state.as_str(),
            motion_device_state.as_str(),
        );
    }

    // Persist only meaningful state changes.
    //
    // Every telemetry reading is still classified, but repeated classifications
    // such as MOVING → MOVING are not written as additional device-state events.
    // Raw telemetry remains available through sensor_readings and telemetry replay.
    if state_changed {
        repository::create_device_state_event(
            db_pool,
            NewDeviceStateEvent {
                device_id,

                // Device-state evidence originates from the physical vibration
                // capability, not from the optional fuel sensor.
                sensor_id: Some(vibration_sensor_id),

                state: motion_device_state.as_str().to_string(),

                recorded_at: reading.timestamp,

                vibration_level: Some(imu_interpretation.vibration_score),
                motion_detected: Some(imu_interpretation.motion_detected),

                distance_meters,
                speed_kmh,

                latitude: Some(reading.latitude),
                longitude: Some(reading.longitude),

                source: "telemetry".to_string(),

                message: Some(format!(
                    "Device state changed to {:?}. Vibration score: {:.2}, \
                movement confidence: {:.2}",
                    motion_device_state,
                    imu_interpretation.vibration_score,
                    imu_interpretation.movement_confidence,
                )),
            },
        )
        .await?;

        println!(
            "[DEVICE STATE EVENT] device_id={}, previous_state={}, current_state={}",
            device_id,
            previous_state
                .as_ref()
                .map(DeviceOperationalState::as_str)
                .unwrap_or("NONE"),
            motion_device_state.as_str(),
        );
    } else {
        println!(
            "[DEVICE STATE UNCHANGED] device_id={}, state={}, event_not_persisted=true",
            device_id,
            motion_device_state.as_str(),
        );
    }

    Ok(())
}

fn calculate_movement(
    reading: &FuelReading,
    previous_position: Option<&StoredTelemetryPosition>,
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let Some(previous_position) = previous_position else {
        return (None, None, None, None);
    };

    let distance_meters = calculate_distance_meters(
        previous_position.latitude,
        previous_position.longitude,
        reading.latitude,
        reading.longitude,
    );

    let time_seconds = (reading.timestamp - previous_position.recorded_at)
        .num_seconds()
        .max(0) as f64;

    let speed_kmh = calculate_speed_kmh(distance_meters, time_seconds);

    (
        Some(previous_position.latitude),
        Some(previous_position.longitude),
        Some(distance_meters),
        Some(speed_kmh),
    )
}
