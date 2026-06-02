import { create } from "zustand";

import {
  createGeofence,
  listGeofences,
  checkPositionAgainstGeofences,
} from "../api/geofenceApi";

import type {
  CreateGeofencePayload,
  Geofence,
  CheckPositionResponse,
} from "../types";

interface GeofenceStore {
  geofences: Geofence[];

  loading: boolean;

  error: string | null;

  loadGeofences: (organizationId: string) => Promise<void>;

  createGeofenceRecord: (payload: CreateGeofencePayload) => Promise<void>;

  positionStatus: CheckPositionResponse | null;

  checkCurrentPosition: (
    organizationId: string,
    deviceId: string,
    latitude: number,
    longitude: number,
  ) => Promise<void>;
}

export const useGeofenceStore = create<GeofenceStore>((set) => ({
  geofences: [],

  loading: false,

  error: null,

  positionStatus: null,

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

  checkCurrentPosition: async (
    organizationId,
    deviceId,
    latitude,
    longitude,
  ) => {
    try {
      const response = await checkPositionAgainstGeofences({
        organization_id: organizationId,
        device_id: deviceId,
        latitude,
        longitude,
      });

      set({
        positionStatus: response,
      });
    } catch (error) {
      console.error("Failed to check position against geofences", error);
    }
  },
}));
