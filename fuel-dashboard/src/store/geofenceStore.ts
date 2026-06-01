import { create } from "zustand";

import { createGeofence, listGeofences } from "../api/geofenceApi";

import type { CreateGeofencePayload, Geofence } from "../types";

interface GeofenceStore {
  geofences: Geofence[];

  loading: boolean;

  error: string | null;

  loadGeofences: (organizationId: string) => Promise<void>;

  createGeofenceRecord: (payload: CreateGeofencePayload) => Promise<void>;
}

export const useGeofenceStore = create<GeofenceStore>((set) => ({
  geofences: [],

  loading: false,

  error: null,

  loadGeofences: async (organizationId: string) => {
    try {
      set({
        loading: true,
        error: null,
      });

      const geofences = await listGeofences(organizationId);

      set({
        geofences,
        loading: false,
      });
    } catch (error) {
      console.error("Failed to load geofences", error);

      set({
        loading: false,
        error: "Failed to load geofences",
      });
    }
  },

  createGeofenceRecord: async (payload: CreateGeofencePayload) => {
    try {
      set({
        loading: true,
        error: null,
      });

      const geofence = await createGeofence(payload);

      set((state) => ({
        geofences: [geofence, ...state.geofences],
        loading: false,
      }));
    } catch (error) {
      console.error("Failed to create geofence", error);

      set({
        loading: false,
        error: "Failed to create geofence",
      });
    }
  },
}));
