-- Add migration script here
ALTER TABLE devices
ADD COLUMN last_seen_at TIMESTAMPTZ,
ADD COLUMN last_heartbeat_at TIMESTAMPTZ,
ADD COLUMN last_payload_at TIMESTAMPTZ,
ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

UPDATE devices
SET status = 'UNKNOWN'
WHERE status = 'active';

ALTER TABLE devices
ALTER COLUMN status SET DEFAULT 'UNKNOWN';

CREATE INDEX idx_devices_status
ON devices(status);

CREATE INDEX idx_devices_last_seen_at
ON devices(last_seen_at);