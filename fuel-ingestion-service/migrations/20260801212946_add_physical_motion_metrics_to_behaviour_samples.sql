-- Add migration script here
ALTER TABLE operational_behaviour_samples
    ADD COLUMN average_gravity_deviation_g DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN average_rotation_magnitude_dps DOUBLE PRECISION NOT NULL DEFAULT 0.0;