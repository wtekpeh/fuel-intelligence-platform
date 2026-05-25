import { create } from "zustand";

import type { OrganizationOverview } from "../types";

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

  setOrganizations: (organizations) =>
    set({
      organizations,
    }),

  selectOrganization: (organization) =>
    set({
      selectedOrganization: organization,
    }),

  clearSelectedOrganization: () =>
    set({
      selectedOrganization: null,
    }),
}));
