-- Add migration script here
CREATE TABLE operational_behaviour_samples (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    learning_session_id UUID NOT NULL
        REFERENCES operational_behaviour_learning_sessions(id)
        ON DELETE CASCADE,

    recorded_at TIMESTAMPTZ NOT NULL,

    vibration_score DOUBLE PRECISION NOT NULL,

    motion_ratio DOUBLE PRECISION NOT NULL,

    average_confidence DOUBLE PRECISION NOT NULL,

    sustained_motion BOOLEAN NOT NULL,

    gps_speed_kmh DOUBLE PRECISION,

    sample_index INTEGER NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CHECK (vibration_score >= 0.0),
    CHECK (motion_ratio >= 0.0 AND motion_ratio <= 1.0),
    CHECK (average_confidence >= 0.0 AND average_confidence <= 1.0),
    CHECK (sample_index > 0)
);

CREATE INDEX idx_operational_behaviour_samples_session
    ON operational_behaviour_samples(learning_session_id);

CREATE INDEX idx_operational_behaviour_samples_recorded_at
    ON operational_behaviour_samples(recorded_at);

CREATE INDEX idx_operational_behaviour_samples_sample_index
    ON operational_behaviour_samples(sample_index);