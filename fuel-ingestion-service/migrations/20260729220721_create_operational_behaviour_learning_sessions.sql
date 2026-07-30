-- Add migration script here
CREATE TABLE operational_behaviour_learning_sessions (
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

    status TEXT NOT NULL
        DEFAULT 'NOT_STARTED'
        CHECK (
            status IN (
                'NOT_STARTED',
                'COLLECTING',
                'COMPLETED',
                'FAILED'
            )
        ),

    requested_sample_count INTEGER NOT NULL
        CHECK (requested_sample_count > 0),

    collected_sample_count INTEGER NOT NULL DEFAULT 0
        CHECK (collected_sample_count >= 0),

    started_at TIMESTAMPTZ,

    completed_at TIMESTAMPTZ,

    failure_reason TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_operational_behaviour_learning_sessions_device
    ON operational_behaviour_learning_sessions(device_id);

CREATE INDEX idx_operational_behaviour_learning_sessions_sensor
    ON operational_behaviour_learning_sessions(sensor_id);

CREATE INDEX idx_operational_behaviour_learning_sessions_status
    ON operational_behaviour_learning_sessions(status);

CREATE INDEX idx_operational_behaviour_learning_sessions_behaviour_type
    ON operational_behaviour_learning_sessions(behaviour_type);