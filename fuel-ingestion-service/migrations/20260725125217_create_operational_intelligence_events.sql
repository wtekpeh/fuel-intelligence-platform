-- Add migration script here
CREATE TABLE operational_intelligence_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    device_id UUID NOT NULL
        REFERENCES devices(id)
        ON DELETE CASCADE,

    operational_transition_event_id UUID
        REFERENCES operational_transition_events(id)
        ON DELETE SET NULL,

    event_type TEXT NOT NULL,

    previous_state TEXT,

    current_state TEXT,

    latitude DOUBLE PRECISION,

    longitude DOUBLE PRECISION,

    recorded_at TIMESTAMPTZ NOT NULL,

    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    source TEXT NOT NULL DEFAULT 'OPERATIONAL_INTELLIGENCE'
);

CREATE INDEX idx_operational_intelligence_events_device
    ON operational_intelligence_events(device_id);

CREATE INDEX idx_operational_intelligence_events_recorded
    ON operational_intelligence_events(recorded_at DESC);

CREATE INDEX idx_operational_intelligence_events_type
    ON operational_intelligence_events(event_type);

CREATE INDEX idx_operational_intelligence_events_transition
    ON operational_intelligence_events(operational_transition_event_id);