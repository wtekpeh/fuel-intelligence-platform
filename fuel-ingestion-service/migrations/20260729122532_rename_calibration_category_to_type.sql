-- Add migration script here
-- ==========================================================
-- Rename calibration_category to calibration_type
-- ==========================================================
--
-- The sensor record already identifies the sensor domain:
-- FUEL, GPS, VIBRATION, TEMPERATURE, etc.
--
-- The calibration field should therefore describe the kind
-- of calibration applied to that sensor instance:
-- INSTALLATION, ZERO_OFFSET, ORIENTATION, GEOMETRY, etc.
-- ==========================================================

ALTER TABLE sensor_calibrations
RENAME COLUMN calibration_category TO calibration_type;

ALTER TABLE sensor_calibrations
RENAME CONSTRAINT sensor_calibrations_category_not_blank
TO sensor_calibrations_type_not_blank;

DROP INDEX IF EXISTS idx_sensor_calibrations_category;

DROP INDEX IF EXISTS idx_sensor_calibrations_sensor_history;

DROP INDEX IF EXISTS unique_active_sensor_calibration;

CREATE INDEX idx_sensor_calibrations_type
ON sensor_calibrations(calibration_type);

CREATE INDEX idx_sensor_calibrations_sensor_history
ON sensor_calibrations(
    sensor_id,
    calibration_type,
    calibrated_at DESC
);

CREATE UNIQUE INDEX unique_active_sensor_calibration
ON sensor_calibrations(
    sensor_id,
    calibration_type
)
WHERE is_active = TRUE;