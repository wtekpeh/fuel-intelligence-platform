-- Add migration script here
ALTER TABLE operational_behaviour_samples
ADD COLUMN motion_sample_count INTEGER NOT NULL DEFAULT 1
CHECK (motion_sample_count > 0);