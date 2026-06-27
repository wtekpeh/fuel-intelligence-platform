import { create } from "zustand";

import { fetchOrganizationOverview } from "../../api/organizationApi";

import type { OrganizationOverview } from "../../types";

interface OrganizationStore {
  organizations: OrganizationOverview[];
  selectedOrganization: OrganizationOverview | null;

  loading: boolean;
  error: string | null;

  loadOrganizations: () => Promise<void>;
  selectOrganization: (organization: OrganizationOverview | null) => void;

  clearError: () => void;
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

  clearError: () => {
    set({ error: null });
  },
}));
