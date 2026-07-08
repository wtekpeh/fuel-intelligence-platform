-- Add migration script here
-- Add migration script here

CREATE TABLE orbi_device_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    device_code TEXT NOT NULL UNIQUE,
    serial_number TEXT NOT NULL UNIQUE,
    imei TEXT UNIQUE,

    device_model_id UUID NOT NULL REFERENCES device_models(id),
    hardware_profile_id UUID NOT NULL REFERENCES hardware_profiles(id),

    firmware_version TEXT,
    production_batch TEXT,

    inventory_status TEXT NOT NULL DEFAULT 'ASSEMBLED',
    quality_test_status TEXT NOT NULL DEFAULT 'PENDING',

    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orbi_device_inventory_device_code
ON orbi_device_inventory(device_code);

CREATE INDEX idx_orbi_device_inventory_serial_number
ON orbi_device_inventory(serial_number);

CREATE INDEX idx_orbi_device_inventory_imei
ON orbi_device_inventory(imei);

CREATE INDEX idx_orbi_device_inventory_inventory_status
ON orbi_device_inventory(inventory_status);

CREATE INDEX idx_orbi_device_inventory_quality_test_status
ON orbi_device_inventory(quality_test_status);

CREATE INDEX idx_orbi_device_inventory_device_model_id
ON orbi_device_inventory(device_model_id);

CREATE INDEX idx_orbi_device_inventory_hardware_profile_id
ON orbi_device_inventory(hardware_profile_id);