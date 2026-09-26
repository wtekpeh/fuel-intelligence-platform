import { create } from "zustand";

import type { OrganizationOverview } from "../types";

const SELECTED_ORGANIZATION_STORAGE_KEY = "orbi:selected-organization-id";

interface OrganizationStore {
  organizations: OrganizationOverview[];
  selectedOrganization: OrganizationOverview | null;

  setOrganizations: (organizations: OrganizationOverview[]) => void;
  selectOrganization: (organization: OrganizationOverview) => void;
  clearSelectedOrganization: () => void;
}

export const useOrganizationStore = create<OrganizationStore>((set) => ({
  organizations: [],
  selectedOrganization: null,

  setOrganizations: (organizations) => {
    const storedOrganizationId = localStorage.getItem(
      SELECTED_ORGANIZATION_STORAGE_KEY,
    );

    const restoredOrganization = storedOrganizationId
      ? (organizations.find(
          (organization) =>
            organization.organization_id === storedOrganizationId,
        ) ?? null)
      : null;

    set({
      organizations,
      selectedOrganization: restoredOrganization,
    });
  },

  selectOrganization: (organization) => {
    localStorage.setItem(
      SELECTED_ORGANIZATION_STORAGE_KEY,
      organization.organization_id,
    );

    set({
      selectedOrganization: organization,
    });
  },

  clearSelectedOrganization: () => {
    localStorage.removeItem(SELECTED_ORGANIZATION_STORAGE_KEY);

    set({
      selectedOrganization: null,
    });
  },
}));
