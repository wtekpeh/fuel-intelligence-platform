use crate::telemetry::record::TelemetryRecord;
use heapless::String;

/*
 * Build the JSON payload sent to the ORBI backend.
 *
 * The payload contains physical measurements only.
 * No operational intelligence is generated inside the firmware.
 */
pub fn build_telemetry_payload(reading: &TelemetryRecord<'_>) -> String<1024> {
    let mut payload = String::<1024>::new();

    let _ = core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
        \"device_id\":\"{}\",\
        \"synced_at\":\"{}\",\
        \"readings\":[{{\
            \"device_id\":\"{}\",\
            \"timestamp\":\"{}\",\
            \"fuel_distance_smooth_cm\":{},\
            \"fuel_distance_realtime_cm\":{},\
            \"fuel_distance_raw_cm\":{},\
            \"fuel_sensor_temperature_c\":{},\
            \"fuel_sensor_status_1\":{},\
            \"fuel_sensor_status_2\":{},\
            \"fuel_raw_data_validity\":{},\
            \"latitude\":{},\
            \"longitude\":{},\
            \"speed\":{},\
            \"heading\":{},\
            \"accel_x_g\":{},\
            \"accel_y_g\":{},\
            \"accel_z_g\":{},\
            \"gyro_x_dps\":{},\
            \"gyro_y_dps\":{},\
            \"gyro_z_dps\":{},\
            \"imu_temperature_c\":{},\
            \"simulation_mode\":\"{}\"\
        }}]\
    }}",
            reading.device_id,
            reading.timestamp,
            reading.device_id,
            reading.timestamp,
            reading.fuel_distance_smooth_cm,
            reading.fuel_distance_realtime_cm,
            reading.fuel_distance_raw_cm,
            reading.fuel_sensor_temperature_c,
            reading.fuel_sensor_status_1,
            reading.fuel_sensor_status_2,
            reading.fuel_raw_data_validity,
            reading.latitude,
            reading.longitude,
            reading.speed,
            reading.heading,
            reading.accel_x_g,
            reading.accel_y_g,
            reading.accel_z_g,
            reading.gyro_x_dps,
            reading.gyro_y_dps,
            reading.gyro_z_dps,
            reading.imu_temperature_c,
            reading.simulation_mode,
        ),
    );

    payload
}

fn extract_json_string<'a>(json: &'a str, field_name: &str) -> Option<&'a str> {
    let mut pattern = String::<64>::new();

    let _ = core::fmt::write(&mut pattern, format_args!("\"{}\":\"", field_name));

    let field_start = json.find(pattern.as_str())?;
    let value_start = field_start + pattern.len();
    let remaining = json.get(value_start..)?;
    let value_end = remaining.find('"')?;

    remaining.get(..value_end)
}

pub fn extract_replay_identity<'a>(queued_record: &'a str) -> Option<(&'a str, &'a str)> {
    let device_id = extract_json_string(queued_record, "device_id")?;
    let timestamp = extract_json_string(queued_record, "timestamp")?;

    Some((device_id, timestamp))
}

/*
 * Build one upload payload from telemetry records already persisted in
 * ORBIQ.LOG.
 *
 * The queued records are complete JSON objects, so they are inserted
 * directly into the `readings` array without being deserialized and rebuilt.
 *
 * This preserves ORBI's offline-first flow:
 *
 * measurement
 *     -> persistent queue
 *     -> queue batch
 *     -> backend
 */
pub fn build_queue_batch_payload<const N: usize>(
    queued_records: &heapless::Vec<heapless::String<768>, N>,
) -> Option<String<4096>> {
    let first_record = queued_records.first()?;

    let (device_id, synced_at) = extract_replay_identity(first_record.as_str())?;

    /*
     * All records in one firmware queue should belong to the same device.
     * Validate that assumption before constructing the batch.
     */
    for queued_record in queued_records {
        let (record_device_id, _) = extract_replay_identity(queued_record.as_str())?;

        if record_device_id != device_id {
            return None;
        }
    }

    let mut payload = String::<4096>::new();

    core::fmt::write(
        &mut payload,
        format_args!(
            "{{\
                \"device_id\":\"{}\",\
                \"synced_at\":\"{}\",\
                \"readings\":[",
            device_id, synced_at,
        ),
    )
    .ok()?;

    for (index, queued_record) in queued_records.iter().enumerate() {
        if index > 0 {
            payload.push(',').ok()?;
        }

        payload.push_str(queued_record.as_str()).ok()?;
    }

    payload.push_str("]}").ok()?;

    Some(payload)
}
