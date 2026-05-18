-- Add migration script here
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    fuel_event_id UUID REFERENCES fuel_events(id) ON DELETE SET NULL,

    alert_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    reason TEXT NOT NULL,

    is_acknowledged BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);