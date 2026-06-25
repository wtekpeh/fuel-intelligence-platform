-- Add migration script here
CREATE TABLE hardware_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    profile_code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE hardware_profile_sensors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    hardware_profile_id UUID NOT NULL REFERENCES hardware_profiles(id) ON DELETE CASCADE,
    sensor_type TEXT NOT NULL,
    unit TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(hardware_profile_id, sensor_type)
);

ALTER TABLE devices
ADD COLUMN hardware_profile_id UUID REFERENCES hardware_profiles(id);

INSERT INTO hardware_profiles (profile_code, name, description)
VALUES
(
    'GPS_ONLY',
    'GPS Only Tracker',
    'Device profile for GPS-only tracking deployments.'
),
(
    'FUEL_FULL',
    'Fuel Intelligence Kit',
    'Device profile for fuel, GPS, and vibration intelligence deployments.'
);

INSERT INTO hardware_profile_sensors (hardware_profile_id, sensor_type, unit)
SELECT id, 'GPS', 'coordinates'
FROM hardware_profiles
WHERE profile_code = 'GPS_ONLY';

INSERT INTO hardware_profile_sensors (hardware_profile_id, sensor_type, unit)
SELECT id, 'GPS', 'coordinates'
FROM hardware_profiles
WHERE profile_code = 'FUEL_FULL';

INSERT INTO hardware_profile_sensors (hardware_profile_id, sensor_type, unit)
SELECT id, 'FUEL', 'litres'
FROM hardware_profiles
WHERE profile_code = 'FUEL_FULL';

INSERT INTO hardware_profile_sensors (hardware_profile_id, sensor_type, unit)
SELECT id, 'VIBRATION', 'level'
FROM hardware_profiles
WHERE profile_code = 'FUEL_FULL';

UPDATE devices
SET hardware_profile_id = (
    SELECT id
    FROM hardware_profiles
    WHERE profile_code = 'FUEL_FULL'
)
WHERE hardware_profile_id IS NULL;

ALTER TABLE devices
ALTER COLUMN hardware_profile_id SET NOT NULL;