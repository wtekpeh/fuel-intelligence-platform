import { create } from "zustand";

import type { OrganizationFleetOverview } from "../types";

interface FleetStore {
  fleetItems: OrganizationFleetOverview[];

  selectedDevice: OrganizationFleetOverview | null;

  setFleetItems: (items: OrganizationFleetOverview[]) => void;

  selectDevice: (device: OrganizationFleetOverview) => void;

  clearSelectedDevice: () => void;
}

export const useFleetStore = create<FleetStore>((set) => ({
  fleetItems: [],

  selectedDevice: null,

  setFleetItems: (items) =>
    set({
      fleetItems: items,
    }),

  selectDevice: (device) =>
    set({
      selectedDevice: device,
    }),

  clearSelectedDevice: () =>
    set({
      selectedDevice: null,
    }),
}));
