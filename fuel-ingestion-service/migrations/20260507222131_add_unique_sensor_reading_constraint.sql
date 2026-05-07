-- Add migration script here
ALTER TABLE sensor_readings
ADD CONSTRAINT unique_sensor_reading_per_time
UNIQUE (sensor_id, recorded_at);