import { create } from "zustand";

import {
  createOrganization,
  fetchOrganizationOverview,
  type CreateOrganizationRequest,
} from "../../api/organizationApi";

import type { OrganizationOverview } from "../../types";

/*
 * Temporary development tenant.
 *
 * ORBI currently has no authentication/tenant identity provider wired into
 * the dashboard. During physical hardware bench testing, the platform should
 * therefore enter the ORBI test organization rather than whichever
 * organization happens to be returned first by the API.
 *
 * This must be removed once Keycloak-backed organization context becomes
 * authoritative.
 */
const DEVELOPMENT_ORGANIZATION_ID = "fad51a33-2cbd-4bea-a195-6936e02f3e05";

interface OrganizationStore {
  organizations: OrganizationOverview[];
  selectedOrganization: OrganizationOverview | null;

  loading: boolean;
  error: string | null;

  loadOrganizations: () => Promise<void>;
  selectOrganization: (organization: OrganizationOverview | null) => void;

  clearError: () => void;

  createOrganization: (
    request: CreateOrganizationRequest,
  ) => Promise<string | null>;
}

export const useOrganizationStore = create<OrganizationStore>((set) => ({
  organizations: [],
  selectedOrganization: null,

  loading: false,
  error: null,

  loadOrganizations: async () => {
    set({
      loading: true,
      error: null,
    });

    try {
      const organizations = await fetchOrganizationOverview();

      const developmentOrganization =
        organizations.find(
          (organization) =>
            organization.organization_id === DEVELOPMENT_ORGANIZATION_ID,
        ) ??
        organizations[0] ??
        null;

      set({
        organizations,
        selectedOrganization: developmentOrganization,
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to load organizations.",
      });
    }
  },

  selectOrganization: (organization) => {
    set({
      selectedOrganization: organization,
    });
  },

  createOrganization: async (request) => {
    set({ loading: true, error: null });

    try {
      const result = await createOrganization(request);
      const organizations = await fetchOrganizationOverview();

      const createdOrganization =
        organizations.find(
          (organization) =>
            organization.organization_id === result.organization_id,
        ) ??
        organizations[0] ??
        null;

      set({
        organizations,
        selectedOrganization: createdOrganization,
        loading: false,
      });

      return result.organization_id;
    } catch {
      set({
        loading: false,
        error: "Failed to create organization.",
      });

      return null;
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));
