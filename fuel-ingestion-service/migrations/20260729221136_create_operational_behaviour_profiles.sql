-- Add migration script here
CREATE TABLE operational_behaviour_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    device_id UUID NOT NULL
        REFERENCES devices(id)
        ON DELETE CASCADE,

    sensor_id UUID NOT NULL
        REFERENCES sensors(id)
        ON DELETE CASCADE,

    behaviour_type TEXT NOT NULL
        CHECK (
            behaviour_type IN (
                'PARKED',
                'IDLE',
                'MOVING'
            )
        ),

    learning_session_id UUID NOT NULL
        REFERENCES operational_behaviour_learning_sessions(id)
        ON DELETE RESTRICT,

    sample_count INTEGER NOT NULL
        CHECK (sample_count > 0),

    average_vibration_score DOUBLE PRECISION NOT NULL,

    minimum_vibration_score DOUBLE PRECISION NOT NULL,

    maximum_vibration_score DOUBLE PRECISION NOT NULL,

    vibration_standard_deviation DOUBLE PRECISION NOT NULL,

    average_motion_ratio DOUBLE PRECISION NOT NULL
        CHECK (
            average_motion_ratio >= 0.0
            AND average_motion_ratio <= 1.0
        ),

    average_confidence DOUBLE PRECISION NOT NULL
        CHECK (
            average_confidence >= 0.0
            AND average_confidence <= 1.0
        ),

    learned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (
        device_id,
        sensor_id,
        behaviour_type
    )
);

CREATE INDEX idx_operational_behaviour_profiles_device
    ON operational_behaviour_profiles(device_id);

CREATE INDEX idx_operational_behaviour_profiles_sensor
    ON operational_behaviour_profiles(sensor_id);

CREATE INDEX idx_operational_behaviour_profiles_behaviour
    ON operational_behaviour_profiles(behaviour_type);