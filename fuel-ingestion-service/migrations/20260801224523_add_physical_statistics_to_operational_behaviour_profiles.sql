-- Add migration script here
ALTER TABLE operational_behaviour_profiles
    ADD COLUMN average_gravity_deviation_g DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN minimum_gravity_deviation_g DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN maximum_gravity_deviation_g DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN gravity_deviation_variance DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN gravity_deviation_standard_deviation DOUBLE PRECISION NOT NULL DEFAULT 0.0,

    ADD COLUMN average_rotation_magnitude_dps DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN minimum_rotation_magnitude_dps DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN maximum_rotation_magnitude_dps DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN rotation_magnitude_variance DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN rotation_magnitude_standard_deviation DOUBLE PRECISION NOT NULL DEFAULT 0.0;