-- Add migration script here
ALTER TABLE alerts
ADD COLUMN status TEXT NOT NULL DEFAULT 'OPEN';