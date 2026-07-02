import { create } from "zustand";

import {
  createOrganization,
  fetchOrganizationOverview,
  type CreateOrganizationRequest,
} from "../../api/organizationApi";

import type { OrganizationOverview } from "../../types";

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

      set({
        organizations,
        selectedOrganization: organizations[0] ?? null,
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
