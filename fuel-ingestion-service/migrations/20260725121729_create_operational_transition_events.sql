-- Add migration script here
CREATE TABLE operational_transition_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    device_id UUID NOT NULL
        REFERENCES devices(id)
        ON DELETE CASCADE,

    previous_state TEXT NOT NULL,

    current_state TEXT NOT NULL,

    transition TEXT NOT NULL,

    latitude DOUBLE PRECISION,

    longitude DOUBLE PRECISION,

    recorded_at TIMESTAMPTZ NOT NULL,

    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    source TEXT NOT NULL DEFAULT 'MOTION_PIPELINE'
);

CREATE INDEX idx_operational_transition_events_device
    ON operational_transition_events(device_id);

CREATE INDEX idx_operational_transition_events_recorded
    ON operational_transition_events(recorded_at DESC);

CREATE INDEX idx_operational_transition_events_transition
    ON operational_transition_events(transition);