-- Add migration script here
ALTER TABLE organizations
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;