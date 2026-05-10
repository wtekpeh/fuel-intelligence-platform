-- Add migration script here
CREATE TABLE fuel_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    sensor_id UUID NOT NULL REFERENCES sensors(id) ON DELETE CASCADE,

    event_type TEXT NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    fuel_before DOUBLE PRECISION NOT NULL,
    fuel_after DOUBLE PRECISION NOT NULL,
    fuel_difference DOUBLE PRECISION NOT NULL,
    duration_seconds BIGINT NOT NULL,

    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,

    is_delayed_detection BOOLEAN NOT NULL DEFAULT FALSE,
    sync_delay_seconds BIGINT NOT NULL DEFAULT 0,

    severity TEXT NOT NULL,
    message TEXT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fuel_events_device_event_time
ON fuel_events(device_id, event_time);

CREATE INDEX idx_fuel_events_sensor_event_time
ON fuel_events(sensor_id, event_time);