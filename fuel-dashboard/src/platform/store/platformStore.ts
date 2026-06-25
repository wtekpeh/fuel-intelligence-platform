import { create } from "zustand";
import {
  fetchDeviceSensors,
  fetchDevices,
  fetchHardwareProfileSensors,
  fetchHardwareProfiles,
  registerDevice,
} from "../api/platformApi";
import type {
  DeviceSensorSummary,
  DeviceSummary,
  HardwareProfile,
  HardwareProfileSensor,
  RegisterDeviceRequest,
} from "../types";

interface PlatformStore {
  hardwareProfiles: HardwareProfile[];
  selectedHardwareProfile: HardwareProfile | null;
  hardwareProfileSensors: HardwareProfileSensor[];

  devices: DeviceSummary[];
  selectedDevice: DeviceSummary | null;
  deviceSensors: DeviceSensorSummary[];

  loading: boolean;
  error: string | null;

  loadHardwareProfiles: () => Promise<void>;
  selectHardwareProfile: (profile: HardwareProfile) => Promise<void>;

  loadDevices: () => Promise<void>;
  selectDevice: (device: DeviceSummary) => Promise<void>;

  createDevice: (payload: RegisterDeviceRequest) => Promise<void>;

  clearError: () => void;
}

export const usePlatformStore = create<PlatformStore>((set, get) => ({
  hardwareProfiles: [],
  selectedHardwareProfile: null,
  hardwareProfileSensors: [],

  devices: [],
  selectedDevice: null,
  deviceSensors: [],

  loading: false,
  error: null,

  loadHardwareProfiles: async () => {
    set({ loading: true, error: null });

    try {
      const profiles = await fetchHardwareProfiles();
      set({ hardwareProfiles: profiles, loading: false });
    } catch {
      set({
        error: "Failed to load hardware profiles.",
        loading: false,
      });
    }
  },

  selectHardwareProfile: async (profile) => {
    set({
      selectedHardwareProfile: profile,
      hardwareProfileSensors: [],
      loading: true,
      error: null,
    });

    try {
      const sensors = await fetchHardwareProfileSensors(profile.id);
      set({
        hardwareProfileSensors: sensors,
        loading: false,
      });
    } catch {
      set({
        error: "Failed to load hardware profile sensors.",
        loading: false,
      });
    }
  },

  loadDevices: async () => {
    set({ loading: true, error: null });

    try {
      const devices = await fetchDevices();
      set({ devices, loading: false });
    } catch {
      set({
        error: "Failed to load devices.",
        loading: false,
      });
    }
  },

  selectDevice: async (device) => {
    set({
      selectedDevice: device,
      deviceSensors: [],
      loading: true,
      error: null,
    });

    try {
      const sensors = await fetchDeviceSensors(device.id);
      set({
        deviceSensors: sensors,
        loading: false,
      });
    } catch {
      set({
        error: "Failed to load device sensors.",
        loading: false,
      });
    }
  },

  createDevice: async (payload) => {
    set({ loading: true, error: null });

    try {
      await registerDevice(payload);
      await get().loadDevices();

      set({ loading: false });
    } catch {
      set({
        error: "Failed to register device.",
        loading: false,
      });
    }
  },

  clearError: () => set({ error: null }),
}));
