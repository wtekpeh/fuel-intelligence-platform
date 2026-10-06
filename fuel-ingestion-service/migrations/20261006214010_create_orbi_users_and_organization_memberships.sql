-- Add migration script here
-- Human identities known to the ORBI application.
--
-- Authentication itself is owned by Keycloak. The immutable Keycloak
-- subject (`sub`) links an authenticated Keycloak identity to its ORBI
-- application account.
CREATE TABLE orbi_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    keycloak_subject TEXT NOT NULL UNIQUE,

    username TEXT,
    email TEXT,

    -- Platform-wide authority.
    --
    -- SUPER_ADMIN:
    --     Full ORBI platform administration.
    --
    -- STAFF:
    --     Internal ORBI staff. Fine-grained staff permissions can be
    --     introduced later without changing organization membership.
    platform_role TEXT,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_orbi_users_platform_role
        CHECK (
            platform_role IS NULL
            OR platform_role IN ('SUPER_ADMIN', 'STAFF')
        )
);


-- Authorization relationship between an ORBI user and a customer
-- organization.
--
-- A user's authority inside one organization is deliberately separate
-- from their platform-wide ORBI role.
CREATE TABLE organization_memberships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    user_id UUID NOT NULL
        REFERENCES orbi_users(id)
        ON DELETE CASCADE,

    organization_id UUID NOT NULL
        REFERENCES organizations(id)
        ON DELETE CASCADE,

    role TEXT NOT NULL,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_organization_memberships_role
        CHECK (
            role IN (
                'ADMIN',
                'FLEET_MANAGER',
                'OPERATOR',
                'VIEWER'
            )
        ),

    CONSTRAINT uq_organization_memberships_user_organization
        UNIQUE (user_id, organization_id)
);


CREATE INDEX idx_organization_memberships_user_id
ON organization_memberships(user_id);


CREATE INDEX idx_organization_memberships_organization_id
ON organization_memberships(organization_id);