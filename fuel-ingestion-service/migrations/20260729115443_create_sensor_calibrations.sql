-- Add migration script here
-- ==========================================================
-- Sensor Calibration Foundation
-- ==========================================================
--
-- Stores installation-specific calibration history for
-- provisioned sensor instances.
--
-- Examples:
--
-- VIBRATION:
-- {
--     "idle_vibration_threshold": 0.53
-- }
--
-- FUEL:
-- {
--     "empty_distance_cm": 145.0,
--     "full_distance_cm": 12.0,
--     "tank_capacity_litres": 600.0
-- }
--
-- Only one active calibration is allowed for a given
-- sensor and calibration category.
-- ==========================================================

CREATE TABLE sensor_calibrations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    sensor_id UUID NOT NULL
        REFERENCES sensors(id)
        ON DELETE CASCADE,

    calibration_category TEXT NOT NULL,

    calibration_values JSONB NOT NULL,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    calibrated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT sensor_calibrations_category_not_blank
        CHECK (BTRIM(calibration_category) <> ''),

    CONSTRAINT sensor_calibrations_values_is_object
        CHECK (jsonb_typeof(calibration_values) = 'object')
);

CREATE INDEX idx_sensor_calibrations_sensor_id
ON sensor_calibrations(sensor_id);

CREATE INDEX idx_sensor_calibrations_category
ON sensor_calibrations(calibration_category);

CREATE INDEX idx_sensor_calibrations_sensor_history
ON sensor_calibrations(
    sensor_id,
    calibration_category,
    calibrated_at DESC
);

CREATE UNIQUE INDEX unique_active_sensor_calibration
ON sensor_calibrations(
    sensor_id,
    calibration_category
)
WHERE is_active = TRUE;