-- Add migration script here
-- ==========================================================
-- Support Abandoned Guided Fuel Calibration Sessions
-- ==========================================================
--
-- A guided fuel-calibration session may be abandoned when the
-- calibration attempt is intentionally stopped and must not be
-- resumed or completed later.
--
-- Abandoned sessions remain in the database for audit/history.
--
-- Lifecycle:
--
-- active
-- paused
-- abandoned
-- completed
--
-- completed_at remains NULL for active, paused and abandoned
-- sessions. Only completed sessions receive completed_at.
-- ==========================================================


ALTER TABLE fuel_calibration_sessions
DROP CONSTRAINT fuel_calibration_sessions_status_check;


ALTER TABLE fuel_calibration_sessions
ADD CONSTRAINT fuel_calibration_sessions_status_check
CHECK (
    status IN (
        'active',
        'paused',
        'abandoned',
        'completed'
    )
);


ALTER TABLE fuel_calibration_sessions
DROP CONSTRAINT fuel_calibration_session_completion_time;


ALTER TABLE fuel_calibration_sessions
ADD CONSTRAINT fuel_calibration_session_completion_time
CHECK (
    (
        status IN (
            'active',
            'paused',
            'abandoned'
        )
        AND completed_at IS NULL
    )
    OR
    (
        status = 'completed'
        AND completed_at IS NOT NULL
    )
);