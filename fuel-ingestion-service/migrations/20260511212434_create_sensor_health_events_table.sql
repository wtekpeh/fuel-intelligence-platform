-- Add migration script here
CREATE TABLE sensor_health_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    sensor_id UUID NOT NULL REFERENCES sensors(id) ON DELETE CASCADE,

    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    reason TEXT NOT NULL,

    first_seen_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sensor_health_events_sensor_detected_at
ON sensor_health_events(sensor_id, detected_at);

CREATE INDEX idx_sensor_health_events_device_detected_at
ON sensor_health_events(device_id, detected_at);

CREATE INDEX idx_sensor_health_events_event_type
ON sensor_health_events(event_type);