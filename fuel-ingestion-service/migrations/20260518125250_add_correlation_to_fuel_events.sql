-- Add migration script here
ALTER TABLE fuel_events
ADD COLUMN correlation_status TEXT,
ADD COLUMN correlation_reason TEXT;