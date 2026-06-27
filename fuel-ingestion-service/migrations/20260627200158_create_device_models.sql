-- Add migration script here
CREATE TABLE device_models (
    id UUID PRIMARY KEY,
    model_code TEXT NOT NULL UNIQUE,
    model_name TEXT NOT NULL,
    manufacturer TEXT,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE devices
ADD COLUMN device_model_id UUID REFERENCES device_models(id);

INSERT INTO device_models (
    id,
    model_code,
    model_name,
    manufacturer,
    description
)
VALUES
(
    gen_random_uuid(),
    'ORBI-A100',
    'ORBI A100',
    'ORBI',
    'General-purpose telemetry unit for GPS and fuel intelligence deployments.'
),
(
    gen_random_uuid(),
    'ORBI-GPS-LITE',
    'ORBI GPS Lite',
    'ORBI',
    'GPS-only tracking device for route, geofence, and journey intelligence.'
),
(
    gen_random_uuid(),
    'ORBI-FULL-KIT',
    'ORBI Full Intelligence Kit',
    'ORBI',
    'Fuel, GPS, and vibration telemetry kit for full operational intelligence.'
);