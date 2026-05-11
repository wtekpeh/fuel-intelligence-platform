-- Add migration script here
CREATE TABLE device_health_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    previous_status TEXT,
    new_status TEXT NOT NULL,

    reason TEXT NOT NULL,

    last_seen_at TIMESTAMPTZ,
    last_heartbeat_at TIMESTAMPTZ,
    last_payload_at TIMESTAMPTZ,

    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_device_health_events_device_detected_at
ON device_health_events(device_id, detected_at);

CREATE INDEX idx_device_health_events_new_status
ON device_health_events(new_status);