import { create } from "zustand";
import { getAlertTrends, getGeofenceActivityTrends } from "../api/analyticsApi";
import type {
  AlertTrendsResponse,
  GeofenceActivityTrendResponse,
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

  fetchGeofenceActivityTrends: (days?: number) => Promise<void>;

  fetchAlertTrends: (days?: number) => Promise<void>;
}

export const useAnalyticsStore = create<AnalyticsStore>((set) => ({
  alertTrends: null,
  selectedDays: 30,

  setSelectedDays: (days: number) => {
    set({
      selectedDays: days,
    });
  },
  loadingAlertTrends: false,
  alertTrendsError: null,
  geofenceActivityTrends: null,
  loadingGeofenceActivityTrends: false,
  geofenceActivityTrendsError: null,

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
}));
