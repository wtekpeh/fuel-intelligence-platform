-- Add migration script here
UPDATE alerts
SET status = 'ACKNOWLEDGED'
WHERE is_acknowledged = true
  AND status = 'OPEN';