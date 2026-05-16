-- Add migration script here
CREATE TABLE device_state_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    sensor_id UUID REFERENCES sensors(id) ON DELETE SET NULL,

    state TEXT NOT NULL,

    recorded_at TIMESTAMPTZ NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    vibration_level DOUBLE PRECISION,
    motion_detected BOOLEAN,

    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,

    source TEXT NOT NULL DEFAULT 'telemetry',
    message TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_device_state_events_device_id
ON device_state_events(device_id);

CREATE INDEX idx_device_state_events_recorded_at
ON device_state_events(recorded_at);

CREATE INDEX idx_device_state_events_state
ON device_state_events(state);