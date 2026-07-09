-- Add migration script here
-- ==========================================================
-- Add ORBI GPS Control Kit
-- ==========================================================

-------------------------------------------------------------
-- 1. Create GPS Control hardware profile
-------------------------------------------------------------

INSERT INTO hardware_profiles (
    id,
    profile_code,
    name,
    description,
    is_active
)
VALUES (
    gen_random_uuid(),
    'GPS_CONTROL',
    'GPS Control Kit',
    'GPS tracking with remote kill switch.',
    TRUE
);

-------------------------------------------------------------
-- 2. GPS capability
-------------------------------------------------------------

INSERT INTO hardware_profile_sensors (
    hardware_profile_id,
    sensor_type,
    unit
)
SELECT
    id,
    'GPS',
    'coordinates'
FROM hardware_profiles
WHERE profile_code = 'GPS_CONTROL';

-------------------------------------------------------------
-- 3. Kill Switch capability
-------------------------------------------------------------

INSERT INTO hardware_profile_sensors (
    hardware_profile_id,
    sensor_type,
    unit
)
SELECT
    id,
    'KILL_SWITCH',
    'state'
FROM hardware_profiles
WHERE profile_code = 'GPS_CONTROL';

-------------------------------------------------------------
-- 4. Create device model
-------------------------------------------------------------

INSERT INTO device_models (
    id,
    model_code,
    model_name,
    manufacturer,
    description,
    is_active
)
VALUES (
    gen_random_uuid(),
    'ORBI-GPS-CONTROL',
    'ORBI GPS Control Kit',
    'ORBI',
    'GPS tracking with remote immobilization.',
    TRUE
);

-------------------------------------------------------------
-- 5. Link model to profile
-------------------------------------------------------------

INSERT INTO device_model_hardware_profiles (
    device_model_id,
    hardware_profile_id,
    is_default
)
SELECT
    dm.id,
    hp.id,
    TRUE
FROM device_models dm
JOIN hardware_profiles hp
    ON hp.profile_code = 'GPS_CONTROL'
WHERE dm.model_code = 'ORBI-GPS-CONTROL';