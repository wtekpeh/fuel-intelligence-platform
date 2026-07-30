-- Add migration script here
ALTER TABLE operational_behaviour_profiles
    ADD COLUMN vibration_variance DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN minimum_motion_ratio DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN maximum_motion_ratio DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN sustained_motion_ratio DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ADD COLUMN average_gps_speed_kmh DOUBLE PRECISION;

ALTER TABLE operational_behaviour_profiles
    ADD CONSTRAINT operational_behaviour_profiles_sustained_motion_ratio_check
    CHECK (
        sustained_motion_ratio >= 0.0
        AND sustained_motion_ratio <= 1.0
    );