-- Add migration script here
-- ==========================================================
-- Refine ORBI Product Catalogue
-- ==========================================================

-- ----------------------------------------------------------
-- 1. Rename the existing Fuel profile
-- ----------------------------------------------------------
ALTER TABLE hardware_profiles
ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT TRUE;

UPDATE hardware_profiles
SET
    profile_code = 'FUEL_INTELLIGENCE',
    name = 'Fuel Intelligence Kit',
    description = 'Fuel, GPS and vibration operational intelligence profile.'
WHERE profile_code = 'FUEL_FULL';


-- ----------------------------------------------------------
-- 2. Rename ORBI-A100 to ORBI Fuel Intelligence Kit
-- ----------------------------------------------------------

UPDATE device_models
SET
    model_code = 'ORBI-FUEL-KIT',
    model_name = 'ORBI Fuel Intelligence Kit',
    description = 'Fuel, GPS and vibration operational intelligence device.'
WHERE model_code = 'ORBI-A100';


-- ----------------------------------------------------------
-- 3. Create Full Intelligence hardware profile
-- ----------------------------------------------------------

INSERT INTO hardware_profiles (
    id,
    profile_code,
    name,
    description,
    is_active
)
VALUES (
    gen_random_uuid(),
    'FULL_INTELLIGENCE',
    'Full Intelligence Kit',
    'Fuel, GPS, vibration and remote kill switch.',
    TRUE
);


-- ----------------------------------------------------------
-- 4. Copy Fuel Intelligence sensors into Full Intelligence
-- ----------------------------------------------------------

INSERT INTO hardware_profile_sensors (
    hardware_profile_id,
    sensor_type,
    unit
)
SELECT
    full_profile.id,
    sensor.sensor_type,
    sensor.unit
FROM hardware_profile_sensors sensor
JOIN hardware_profiles fuel_profile
    ON fuel_profile.id = sensor.hardware_profile_id
JOIN hardware_profiles full_profile
    ON full_profile.profile_code = 'FULL_INTELLIGENCE'
WHERE fuel_profile.profile_code = 'FUEL_INTELLIGENCE';


-- ----------------------------------------------------------
-- 5. Add Remote Kill Switch capability
--
-- NOTE:
-- For MVP we model this as another capability using the
-- existing hardware_profile_sensors table.
-- Later this table will evolve into hardware_profile_capabilities.
-- ----------------------------------------------------------

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
WHERE profile_code = 'FULL_INTELLIGENCE';


-- ----------------------------------------------------------
-- 6. Associate ORBI Full Intelligence Kit with the new profile
-- ----------------------------------------------------------

UPDATE device_model_hardware_profiles
SET hardware_profile_id = (
    SELECT id
    FROM hardware_profiles
    WHERE profile_code = 'FULL_INTELLIGENCE'
)
WHERE device_model_id = (
    SELECT id
    FROM device_models
    WHERE model_code = 'ORBI-FULL-KIT'
);