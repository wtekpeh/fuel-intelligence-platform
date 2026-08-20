-- ==========================================================
-- Guided Fuel Calibration Workflow
-- ==========================================================
--
-- Platform Management owns this workflow.
--
-- These tables store progressive installation-specific fuel
-- calibration work, including sessions that may be paused and
-- resumed later.
--
-- A completed and approved calibration may subsequently be
-- published into sensor_calibrations for consumption by
-- Operational Intelligence.
--
-- sensor_calibrations therefore remains the runtime source of
-- active calibration while these tables manage the calibration
-- process itself.
-- ==========================================================


-- ==========================================================
-- Fuel Calibration Profiles
-- ==========================================================

CREATE TABLE fuel_calibration_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    sensor_id UUID NOT NULL
        REFERENCES sensors(id)
        ON DELETE CASCADE,

    tank_capacity_litres DOUBLE PRECISION NOT NULL
        CHECK (
            tank_capacity_litres > 0
            AND tank_capacity_litres <> 'Infinity'::float8
            AND tank_capacity_litres <> '-Infinity'::float8
            AND tank_capacity_litres <> 'NaN'::float8
        ),

    status TEXT NOT NULL
        DEFAULT 'draft'
        CHECK (
            status IN (
                'draft',
                'progressive',
                'validated',
                'production',
                'superseded'
            )
        ),

    confidence TEXT NOT NULL
        DEFAULT 'low'
        CHECK (
            confidence IN (
                'low',
                'medium',
                'high',
                'verified'
            )
        ),

    verified_from_litres DOUBLE PRECISION NOT NULL DEFAULT 0
        CHECK (
            verified_from_litres >= 0
            AND verified_from_litres <> 'Infinity'::float8
            AND verified_from_litres <> '-Infinity'::float8
            AND verified_from_litres <> 'NaN'::float8
        ),

    verified_to_litres DOUBLE PRECISION NOT NULL DEFAULT 0
        CHECK (
            verified_to_litres >= 0
            AND verified_to_litres <> 'Infinity'::float8
            AND verified_to_litres <> '-Infinity'::float8
            AND verified_to_litres <> 'NaN'::float8
        ),

    coverage_percentage DOUBLE PRECISION NOT NULL DEFAULT 0
        CHECK (
            coverage_percentage >= 0
            AND coverage_percentage <= 100
            AND coverage_percentage <> 'Infinity'::float8
            AND coverage_percentage <> '-Infinity'::float8
            AND coverage_percentage <> 'NaN'::float8
        ),

    published_calibration_id UUID
        REFERENCES sensor_calibrations(id)
        ON DELETE SET NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fuel_calibration_profile_verified_range_order
        CHECK (
            verified_to_litres >= verified_from_litres
        ),

    CONSTRAINT fuel_calibration_profile_verified_range_capacity
        CHECK (
            verified_from_litres <= tank_capacity_litres
            AND verified_to_litres <= tank_capacity_litres
        )
);


CREATE INDEX idx_fuel_calibration_profiles_sensor
    ON fuel_calibration_profiles(sensor_id);

CREATE INDEX idx_fuel_calibration_profiles_status
    ON fuel_calibration_profiles(status);


-- Only one non-superseded calibration profile should normally
-- represent the current calibration work for one installed fuel sensor.
CREATE UNIQUE INDEX unique_current_fuel_calibration_profile
    ON fuel_calibration_profiles(sensor_id)
    WHERE status <> 'superseded';


-- ==========================================================
-- Guided Fuel Calibration Sessions
-- ==========================================================

CREATE TABLE fuel_calibration_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    profile_id UUID NOT NULL
        REFERENCES fuel_calibration_profiles(id)
        ON DELETE CASCADE,

    status TEXT NOT NULL
        DEFAULT 'active'
        CHECK (
            status IN (
                'active',
                'paused',
                'completed'
            )
        ),

    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    completed_at TIMESTAMPTZ,

    starting_litres DOUBLE PRECISION NOT NULL
        CHECK (
            starting_litres >= 0
            AND starting_litres <> 'Infinity'::float8
            AND starting_litres <> '-Infinity'::float8
            AND starting_litres <> 'NaN'::float8
        ),

    ending_litres DOUBLE PRECISION NOT NULL
        CHECK (
            ending_litres >= 0
            AND ending_litres <> 'Infinity'::float8
            AND ending_litres <> '-Infinity'::float8
            AND ending_litres <> 'NaN'::float8
        ),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT fuel_calibration_session_litre_order
        CHECK (
            ending_litres >= starting_litres
        ),

    CONSTRAINT fuel_calibration_session_completion_time
        CHECK (
            (
                status IN ('active', 'paused')
                AND completed_at IS NULL
            )
            OR
            (
                status = 'completed'
                AND completed_at IS NOT NULL
            )
        ),

    CONSTRAINT fuel_calibration_session_completion_order
        CHECK (
            completed_at IS NULL
            OR completed_at >= started_at
        )
);


CREATE INDEX idx_fuel_calibration_sessions_profile
    ON fuel_calibration_sessions(profile_id);

CREATE INDEX idx_fuel_calibration_sessions_status
    ON fuel_calibration_sessions(status);


-- A profile may contain many historical completed sessions,
-- but only one unfinished session may exist at a time.
CREATE UNIQUE INDEX unique_unfinished_fuel_calibration_session
    ON fuel_calibration_sessions(profile_id)
    WHERE status IN ('active', 'paused');


-- ==========================================================
-- Verified Calibration Points
-- ==========================================================

CREATE TABLE fuel_calibration_session_points (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    session_id UUID NOT NULL
        REFERENCES fuel_calibration_sessions(id)
        ON DELETE CASCADE,

    level_cm DOUBLE PRECISION NOT NULL
        CHECK (
            level_cm >= 0
            AND level_cm <> 'Infinity'::float8
            AND level_cm <> '-Infinity'::float8
            AND level_cm <> 'NaN'::float8
        ),

    litres DOUBLE PRECISION NOT NULL
        CHECK (
            litres >= 0
            AND litres <> 'Infinity'::float8
            AND litres <> '-Infinity'::float8
            AND litres <> 'NaN'::float8
        ),

    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE INDEX idx_fuel_calibration_session_points_session
    ON fuel_calibration_session_points(session_id);

CREATE INDEX idx_fuel_calibration_session_points_capture_order
    ON fuel_calibration_session_points(
        session_id,
        captured_at ASC
    );


-- Prevent the same verified quantity from being recorded twice
-- inside one guided calibration session.
CREATE UNIQUE INDEX unique_fuel_calibration_session_litres
    ON fuel_calibration_session_points(
        session_id,
        litres
    );