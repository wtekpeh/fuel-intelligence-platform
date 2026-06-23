import { create } from "zustand";
import {
  getAlertTrends,
  getGeofenceActivityTrends,
  getDeviceHealthTrends,
  getGeofenceUtilization,
} from "../api/analyticsApi";

import type {
  AlertTrendsResponse,
  GeofenceActivityTrendResponse,
  DeviceHealthTrendResponse,
  GeofenceUtilizationResponse,
} from "../types";

interface AnalyticsStore {
  selectedDays: number;
  setSelectedDays: (days: number) => void;
  alertTrends: AlertTrendsResponse | null;
  loadingAlertTrends: boolean;
  alertTrendsError: string | null;
  geofenceActivityTrends: GeofenceActivityTrendResponse | null;
  loadingGeofenceActivityTrends: boolean;
  geofenceActivityTrendsError: string | null;
  deviceHealthTrends: DeviceHealthTrendResponse | null;
  loadingDeviceHealthTrends: boolean;
  deviceHealthTrendsError: string | null;
  geofenceUtilization: GeofenceUtilizationResponse | null;
  loadingGeofenceUtilization: boolean;
  geofenceUtilizationError: string | null;

  fetchGeofenceUtilization: (days?: number) => Promise<void>;

  fetchGeofenceActivityTrends: (days?: number) => Promise<void>;

  fetchAlertTrends: (days?: number) => Promise<void>;

  fetchDeviceHealthTrends: (days?: number) => Promise<void>;
}

export const useAnalyticsStore = create<AnalyticsStore>((set) => ({
  alertTrends: null,
  selectedDays: 30,
  deviceHealthTrends: null,
  loadingDeviceHealthTrends: false,
  deviceHealthTrendsError: null,
  loadingAlertTrends: false,
  alertTrendsError: null,
  geofenceActivityTrends: null,
  loadingGeofenceActivityTrends: false,
  geofenceActivityTrendsError: null,
  geofenceUtilization: null,
  loadingGeofenceUtilization: false,
  geofenceUtilizationError: null,

  setSelectedDays: (days: number) => {
    set({
      selectedDays: days,
    });
  },

  fetchAlertTrends: async (days: number = 30) => {
    set({
      loadingAlertTrends: true,
      alertTrendsError: null,
    });

    try {
      const data = await getAlertTrends(days);

      set({
        alertTrends: data,
        loadingAlertTrends: false,
      });
    } catch (error) {
      console.error("Failed to fetch alert trends:", error);

      set({
        loadingAlertTrends: false,
        alertTrendsError: "Failed to fetch alert trends.",
      });
    }
  },

  fetchGeofenceActivityTrends: async (days: number = 30) => {
    set({
      loadingGeofenceActivityTrends: true,
      geofenceActivityTrendsError: null,
    });

    try {
      const data = await getGeofenceActivityTrends(days);

      set({
        geofenceActivityTrends: data,
        loadingGeofenceActivityTrends: false,
      });
    } catch (error) {
      console.error("Failed to fetch geofence activity trends:", error);

      set({
        loadingGeofenceActivityTrends: false,
        geofenceActivityTrendsError:
          "Failed to fetch geofence activity trends.",
      });
    }
  },

  fetchDeviceHealthTrends: async (days: number = 30) => {
    set({
      loadingDeviceHealthTrends: true,
      deviceHealthTrendsError: null,
    });

    try {
      const data = await getDeviceHealthTrends(days);

      set({
        deviceHealthTrends: data,
        loadingDeviceHealthTrends: false,
      });
    } catch (error) {
      console.error("Failed to fetch device health trends:", error);

      set({
        loadingDeviceHealthTrends: false,
        deviceHealthTrendsError: "Failed to fetch device health trends.",
      });
    }
  },

  fetchGeofenceUtilization: async (days: number = 30) => {
    set({
      loadingGeofenceUtilization: true,
      geofenceUtilizationError: null,
    });

    try {
      const data = await getGeofenceUtilization(days);

      set({
        geofenceUtilization: data,
        loadingGeofenceUtilization: false,
      });
    } catch (error) {
      console.error("Failed to fetch geofence utilization:", error);

      set({
        loadingGeofenceUtilization: false,
        geofenceUtilizationError: "Failed to fetch geofence utilization.",
      });
    }
  },
}));
