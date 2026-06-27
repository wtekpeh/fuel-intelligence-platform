-- Add migration script here
ALTER TABLE devices
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;