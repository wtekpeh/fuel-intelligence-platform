-- Add migration script here
ALTER TABLE sensor_readings
ADD COLUMN vibration_level DOUBLE PRECISION,
ADD COLUMN motion_detected BOOLEAN;