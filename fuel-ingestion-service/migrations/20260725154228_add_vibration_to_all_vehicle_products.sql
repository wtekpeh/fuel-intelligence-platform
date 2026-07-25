-- Add migration script here
-- ==========================================================
-- Add Vibration Capability to All ORBI Vehicle Products
-- ==========================================================
--
-- ORBI's vehicle product architecture now defines vibration
-- sensing as a standard capability across all four products:
--
-- GPS_ONLY
--     GPS + VIBRATION
--
-- GPS_CONTROL
--     GPS + VIBRATION + KILL_SWITCH
--
-- FUEL_INTELLIGENCE
--     GPS + VIBRATION + FUEL
--
-- FULL_INTELLIGENCE
--     GPS + VIBRATION + FUEL + KILL_SWITCH
--
-- FUEL_INTELLIGENCE and FULL_INTELLIGENCE already include
-- VIBRATION. This migration corrects GPS_ONLY and GPS_CONTROL.
--
-- It also backfills VIBRATION sensor rows for devices that were
-- provisioned before these hardware profiles were corrected.
-- ==========================================================


-- ----------------------------------------------------------
-- 1. Add VIBRATION to the GPS_ONLY hardware profile
-- ----------------------------------------------------------

INSERT INTO hardware_profile_sensors (
    hardware_profile_id,
    sensor_type,
    unit
)
SELECT
    id,
    'VIBRATION',
    'level'
FROM hardware_profiles
WHERE profile_code = 'GPS_ONLY'
ON CONFLICT (hardware_profile_id, sensor_type)
DO NOTHING;


-- ----------------------------------------------------------
-- 2. Add VIBRATION to the GPS_CONTROL hardware profile
-- ----------------------------------------------------------

INSERT INTO hardware_profile_sensors (
    hardware_profile_id,
    sensor_type,
    unit
)
SELECT
    id,
    'VIBRATION',
    'level'
FROM hardware_profiles
WHERE profile_code = 'GPS_CONTROL'
ON CONFLICT (hardware_profile_id, sensor_type)
DO NOTHING;


-- ----------------------------------------------------------
-- 3. Backfill VIBRATION sensors for existing GPS_ONLY devices
--
-- Provisioning normally creates device sensors from the
-- selected hardware profile. Devices provisioned before this
-- migration only received the capabilities that existed at
-- that time, so they need a VIBRATION sensor added directly.
--
-- The standard sensor convention is:
--
-- sensor_code = vibration
-- sensor_type = VIBRATION
-- unit        = level
-- ----------------------------------------------------------

INSERT INTO sensors (
    device_id,
    sensor_code,
    sensor_type,
    unit
)
SELECT
    d.id,
    'vibration',
    'VIBRATION',
    'level'
FROM devices d
JOIN hardware_profiles hp
    ON hp.id = d.hardware_profile_id
WHERE hp.profile_code = 'GPS_ONLY'
ON CONFLICT (device_id, sensor_code)
DO NOTHING;


-- ----------------------------------------------------------
-- 4. Backfill VIBRATION sensors for existing GPS_CONTROL
--    devices
-- ----------------------------------------------------------

INSERT INTO sensors (
    device_id,
    sensor_code,
    sensor_type,
    unit
)
SELECT
    d.id,
    'vibration',
    'VIBRATION',
    'level'
FROM devices d
JOIN hardware_profiles hp
    ON hp.id = d.hardware_profile_id
WHERE hp.profile_code = 'GPS_CONTROL'
ON CONFLICT (device_id, sensor_code)
DO NOTHING;


-- ----------------------------------------------------------
-- 5. Update hardware-profile descriptions so the catalogue
--    accurately reflects the product capabilities
-- ----------------------------------------------------------

UPDATE hardware_profiles
SET description = 'GPS and vibration operational intelligence profile.'
WHERE profile_code = 'GPS_ONLY';

UPDATE hardware_profiles
SET description = 'GPS, vibration and remote kill switch operational intelligence profile.'
WHERE profile_code = 'GPS_CONTROL';