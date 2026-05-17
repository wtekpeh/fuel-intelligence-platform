-- Add migration script here
ALTER TABLE fuel_events
ADD COLUMN confidence TEXT;