import { create } from "zustand";

import type { OrganizationFleetOverview } from "../types";

const SELECTED_FLEET_DEVICE_STORAGE_KEY = "orbi:selected-fleet-device-id";

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

  setFleetItems: (items) => {
    const storedDeviceId = localStorage.getItem(
      SELECTED_FLEET_DEVICE_STORAGE_KEY,
    );

    const restoredDevice = storedDeviceId
      ? (items.find((item) => item.device_id === storedDeviceId) ?? null)
      : null;

    set({
      fleetItems: items,
      selectedDevice: restoredDevice,
    });
  },

  selectDevice: (device) => {
    localStorage.setItem(SELECTED_FLEET_DEVICE_STORAGE_KEY, device.device_id);

    set({
      selectedDevice: device,
    });
  },

  clearSelectedDevice: () => {
    localStorage.removeItem(SELECTED_FLEET_DEVICE_STORAGE_KEY);

    set({
      selectedDevice: null,
    });
  },
}));
