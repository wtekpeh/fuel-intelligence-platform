-- Add migration script here
CREATE TABLE geofence_transition_events (
    id UUID PRIMARY KEY,

    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    geofence_id UUID NOT NULL REFERENCES geofences(id) ON DELETE CASCADE,

    transition_type TEXT NOT NULL,

    latitude DOUBLE PRECISION NOT NULL,

    longitude DOUBLE PRECISION NOT NULL,

    recorded_at TIMESTAMPTZ NOT NULL,

    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_geofence_transition_events_organization_id
ON geofence_transition_events(organization_id);

CREATE INDEX idx_geofence_transition_events_device_id
ON geofence_transition_events(device_id);

CREATE INDEX idx_geofence_transition_events_geofence_id
ON geofence_transition_events(geofence_id);

CREATE INDEX idx_geofence_transition_events_recorded_at
ON geofence_transition_events(recorded_at);