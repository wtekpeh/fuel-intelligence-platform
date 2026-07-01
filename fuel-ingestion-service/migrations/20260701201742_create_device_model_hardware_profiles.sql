-- Add migration script here
CREATE TABLE device_model_hardware_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_model_id UUID NOT NULL REFERENCES device_models(id),
    hardware_profile_id UUID NOT NULL REFERENCES hardware_profiles(id),
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_device_model_hardware_profile
        UNIQUE (device_model_id, hardware_profile_id)
);

CREATE UNIQUE INDEX unique_default_profile_per_device_model
ON device_model_hardware_profiles (device_model_id)
WHERE is_default = TRUE;

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
    ON hp.profile_code = 'GPS_ONLY'
WHERE dm.model_code = 'ORBI-GPS-LITE';

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
    ON hp.profile_code = 'FUEL_FULL'
WHERE dm.model_code IN (
    'ORBI-A100',
    'ORBI-FULL-KIT'
);