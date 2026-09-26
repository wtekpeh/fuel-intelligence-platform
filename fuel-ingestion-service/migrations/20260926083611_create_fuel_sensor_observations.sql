-- Persist the physical KUM fuel-sensor observations independently from
-- calibrated operational fuel readings.
--
-- This table represents what the hardware physically measured before any
-- tank-specific distance-to-litres calibration is applied.
--
-- Keeping these observations separate from sensor_readings is important:
--
-- fuel_sensor_observations
--     -> physical KUM measurements in centimetres
--
-- sensor_readings
--     -> calibrated operational fuel quantity in litres
--
-- fuel_calibration_session_points
--     -> installer-selected physical observations used as calibration evidence
--
-- Raw observations remain available even when a newly installed fuel sensor
-- does not yet have an active runtime calibration.

CREATE TABLE fuel_sensor_observations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    sensor_id UUID NOT NULL
        REFERENCES sensors(id) ON DELETE CASCADE,

    device_id UUID NOT NULL
        REFERENCES devices(id) ON DELETE CASCADE,

    recorded_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- KUM ultrasonic distance measurements.
    --
    -- realtime_distance_cm is the canonical measurement currently used by
    -- ORBI for runtime fuel calibration because physical hardware testing
    -- showed that it closely follows real liquid-level changes.
    realtime_distance_cm DOUBLE PRECISION NOT NULL,

    -- Retained for calibration stability checks, diagnostics and comparison.
    smooth_distance_cm DOUBLE PRECISION NOT NULL,

    -- Fastest low-level KUM measurement retained for diagnostics.
    raw_distance_cm DOUBLE PRECISION NOT NULL,

    -- Additional physical KUM diagnostics supplied by the firmware.
    temperature_c DOUBLE PRECISION,

    status_1 SMALLINT,
    status_2 SMALLINT,
    raw_data_validity SMALLINT,

    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_fuel_sensor_observation_per_time
        UNIQUE (sensor_id, recorded_at),

    CONSTRAINT fuel_sensor_observation_realtime_non_negative
        CHECK (realtime_distance_cm >= 0),

    CONSTRAINT fuel_sensor_observation_smooth_non_negative
        CHECK (smooth_distance_cm >= 0),

    CONSTRAINT fuel_sensor_observation_raw_non_negative
        CHECK (raw_distance_cm >= 0)
);

-- Supports retrieving the latest physical KUM observation for a particular
-- installed fuel sensor. This will be used by guided calibration when the
-- installer requests a current sensor reading.
CREATE INDEX idx_fuel_sensor_observations_sensor_recorded_at
ON fuel_sensor_observations(sensor_id, recorded_at DESC);

-- Supports device-level physical fuel diagnostics and observation history.
CREATE INDEX idx_fuel_sensor_observations_device_recorded_at
ON fuel_sensor_observations(device_id, recorded_at DESC);