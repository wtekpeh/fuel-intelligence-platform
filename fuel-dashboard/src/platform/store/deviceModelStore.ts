import { create } from "zustand";

import { fetchDeviceModels } from "../../api/platformApi";

import type { DeviceModelResponse } from "../../types";

interface DeviceModelStore {
  deviceModels: DeviceModelResponse[];
  selectedDeviceModel: DeviceModelResponse | null;

  loading: boolean;
  error: string | null;

  loadDeviceModels: () => Promise<void>;
  selectDeviceModel: (model: DeviceModelResponse | null) => void;
}

export const useDeviceModelStore = create<DeviceModelStore>((set) => ({
  deviceModels: [],
  selectedDeviceModel: null,

  loading: false,
  error: null,

  loadDeviceModels: async () => {
    set({ loading: true, error: null });

    try {
      const deviceModels = await fetchDeviceModels();

      set({
        deviceModels,
        selectedDeviceModel: deviceModels[0] ?? null,
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to load device models.",
      });
    }
  },

  selectDeviceModel: (model) => {
    set({ selectedDeviceModel: model });
  },
}));
