import { create } from "zustand";

import {
  fetchHardwareProfiles,
  fetchHardwareProfileSensors,
} from "../../api/platformApi";

import type {
  HardwareProfileResponse,
  HardwareProfileSensorResponse,
} from "../../types";

interface HardwareStore {
  hardwareProfiles: HardwareProfileResponse[];
  selectedHardwareProfile: HardwareProfileResponse | null;
  hardwareProfileSensors: HardwareProfileSensorResponse[];

  loading: boolean;
  error: string | null;

  loadHardwareProfiles: () => Promise<void>;
  selectHardwareProfile: (
    profile: HardwareProfileResponse | null,
  ) => Promise<void>;
}

export const useHardwareStore = create<HardwareStore>((set) => ({
  hardwareProfiles: [],
  selectedHardwareProfile: null,
  hardwareProfileSensors: [],

  loading: false,
  error: null,

  loadHardwareProfiles: async () => {
    set({ loading: true, error: null });

    try {
      const hardwareProfiles = await fetchHardwareProfiles();
      const selectedHardwareProfile = hardwareProfiles[0] ?? null;

      set({
        hardwareProfiles,
        selectedHardwareProfile,
        loading: false,
      });

      if (selectedHardwareProfile) {
        const sensors = await fetchHardwareProfileSensors(
          selectedHardwareProfile.id,
        );

        set({ hardwareProfileSensors: sensors });
      }
    } catch {
      set({
        loading: false,
        error: "Failed to load hardware profiles.",
      });
    }
  },

  selectHardwareProfile: async (profile) => {
    set({
      selectedHardwareProfile: profile,
      hardwareProfileSensors: [],
      error: null,
    });

    if (!profile) {
      return;
    }

    try {
      const sensors = await fetchHardwareProfileSensors(profile.id);

      set({
        hardwareProfileSensors: sensors,
      });
    } catch {
      set({
        error: "Failed to load hardware profile sensors.",
      });
    }
  },
}));
