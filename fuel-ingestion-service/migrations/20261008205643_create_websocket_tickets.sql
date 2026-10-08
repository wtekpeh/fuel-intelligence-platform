-- Add migration script here
-- Short-lived, single-use WebSocket authentication tickets.
-- Shared across ORBI API replicas through PostgreSQL.

CREATE TABLE websocket_tickets (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL
        REFERENCES orbi_users(id)
        ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    expires_at TIMESTAMPTZ NOT NULL,

    CONSTRAINT chk_websocket_ticket_expiration
        CHECK (expires_at > created_at)
);

CREATE INDEX idx_websocket_tickets_expires_at
    ON websocket_tickets (expires_at);