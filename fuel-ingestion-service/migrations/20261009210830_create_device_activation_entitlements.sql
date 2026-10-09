-- Add migration script here
-- Customer device activation entitlements.
--
-- An entitlement authorizes a specific customer organization
-- to activate a specific physical ORBI device.
--
-- Entitlements are assigned by authorized ORBI administrators.
-- Customers cannot create their own entitlements.

CREATE TABLE device_activation_entitlements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    inventory_device_id UUID NOT NULL UNIQUE
        REFERENCES orbi_device_inventory(id),

    organization_id UUID NOT NULL
        REFERENCES organizations(id),

    status TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'ACTIVATED', 'REVOKED')),

    activated_device_id UUID UNIQUE
        REFERENCES devices(id),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    activated_at TIMESTAMPTZ,

    CONSTRAINT valid_activation_state CHECK (
        (
            status = 'ACTIVATED'
            AND activated_device_id IS NOT NULL
            AND activated_at IS NOT NULL
        )
        OR
        (
            status IN ('PENDING', 'REVOKED')
            AND activated_device_id IS NULL
            AND activated_at IS NULL
        )
    )
);

CREATE INDEX idx_device_activation_entitlements_organization
ON device_activation_entitlements(organization_id);

CREATE INDEX idx_device_activation_entitlements_status
ON device_activation_entitlements(status);