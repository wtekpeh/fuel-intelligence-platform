import { create } from "zustand";

import {
  fetchDeviceSensors,
  fetchDevices,
  registerDevice,
} from "../api/platformApi";

import type {
  DeviceSensorSummary,
  DeviceSummary,
  RegisterDeviceRequest,
} from "../types";

interface DeviceStore {
  devices: DeviceSummary[];
  selectedDevice: DeviceSummary | null;
  deviceSensors: DeviceSensorSummary[];

  loading: boolean;
  error: string | null;

  loadDevices: () => Promise<void>;
  selectDevice: (device: DeviceSummary | null) => Promise<void>;
  createDevice: (request: RegisterDeviceRequest) => Promise<void>;

  clearError: () => void;
}

export const useDeviceStore = create<DeviceStore>((set) => ({
  devices: [],
  selectedDevice: null,
  deviceSensors: [],

  loading: false,
  error: null,

  loadDevices: async () => {
    set({
      loading: true,
      error: null,
    });

    try {
      const devices = await fetchDevices();

      set({
        devices,
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to load devices.",
      });
    }
  },

  selectDevice: async (device) => {
    if (!device) {
      set({
        selectedDevice: null,
        deviceSensors: [],
      });

      return;
    }

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
        loading: false,
        error: "Failed to load device sensors.",
      });
    }
  },

  createDevice: async (request) => {
    set({
      loading: true,
      error: null,
    });

    try {
      await registerDevice(request);

      const devices = await fetchDevices();

      set({
        devices,
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to register device.",
      });
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));
