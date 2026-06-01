-- Add migration script here
CREATE TABLE geofences (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    name TEXT NOT NULL,
    geofence_type TEXT NOT NULL,
    geometry GEOMETRY(POLYGON, 4326) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_geofences_organization_id
ON geofences (organization_id);

CREATE INDEX idx_geofences_geometry
ON geofences
USING GIST (geometry);