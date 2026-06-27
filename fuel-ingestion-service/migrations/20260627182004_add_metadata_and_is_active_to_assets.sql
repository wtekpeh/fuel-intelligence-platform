-- Add migration script here
ALTER TABLE assets
ADD COLUMN metadata JSONB,
ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE;