-- Add migration script here
CREATE TABLE device_operational_state_candidates (
    device_id UUID PRIMARY KEY
        REFERENCES devices(id)
        ON DELETE CASCADE,

    candidate_state TEXT NOT NULL
        CHECK (
            candidate_state IN (
                'MOVING',
                'IDLE',
                'PARKED',
                'OFFLINE',
                'UNKNOWN'
            )
        ),

    observation_count INTEGER NOT NULL DEFAULT 1
        CHECK (observation_count > 0),

    first_observed_at TIMESTAMPTZ NOT NULL,

    last_observed_at TIMESTAMPTZ NOT NULL,

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE device_operational_state_candidates IS
'Stores the current unconfirmed operational-state candidate for each device.';

COMMENT ON COLUMN device_operational_state_candidates.candidate_state IS
'Latest operational state repeatedly suggested by the classifier but not yet confirmed.';

COMMENT ON COLUMN device_operational_state_candidates.observation_count IS
'Number of consecutive classifier observations supporting the candidate state.';

COMMENT ON COLUMN device_operational_state_candidates.first_observed_at IS
'Telemetry timestamp when the current candidate state was first observed.';

COMMENT ON COLUMN device_operational_state_candidates.last_observed_at IS
'Telemetry timestamp of the latest observation supporting the candidate state.';