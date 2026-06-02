-- Add migration script here
CREATE TABLE geofence_device_assignments (
    id UUID PRIMARY KEY,

    geofence_id UUID NOT NULL REFERENCES geofences(id) ON DELETE CASCADE,

    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,

    is_included BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (geofence_id, device_id)
);

CREATE INDEX idx_geofence_device_assignments_geofence_id
ON geofence_device_assignments(geofence_id);

CREATE INDEX idx_geofence_device_assignments_device_id
ON geofence_device_assignments(device_id);