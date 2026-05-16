-- Add migration script here
ALTER TABLE device_state_events
ADD COLUMN distance_meters DOUBLE PRECISION,
ADD COLUMN speed_kmh DOUBLE PRECISION;