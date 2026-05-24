import { create } from "zustand";

import type { AlertLifecycleResponse, AlertResponse } from "../types";

interface AlertStore {
  alerts: AlertResponse[];

  selectedAlert: AlertResponse | null;

  highlightedAlertId: string | null;

  setAlerts: (alerts: AlertResponse[]) => void;

  addAlert: (alert: AlertResponse) => void;

  updateAlertLifecycle: (lifecycleUpdate: AlertLifecycleResponse) => void;

  setSelectedAlert: (alert: AlertResponse | null) => void;

  setHighlightedAlertId: (alertId: string | null) => void;
}

export const useAlertStore = create<AlertStore>((set) => ({
  alerts: [],

  selectedAlert: null,

  highlightedAlertId: null,

  setAlerts: (alerts) =>
    set({
      alerts,
    }),

  addAlert: (alert) =>
    set((state) => ({
      alerts: [alert, ...state.alerts],
    })),

  updateAlertLifecycle: (lifecycleUpdate) =>
    set((state) => ({
      alerts: state.alerts.map((alert) => {
        if (alert.id !== lifecycleUpdate.id) {
          return alert;
        }

        return {
          ...alert,
          is_acknowledged: lifecycleUpdate.is_acknowledged,
          status: lifecycleUpdate.status,
        };
      }),
    })),

  setSelectedAlert: (alert) =>
    set({
      selectedAlert: alert,
    }),

  setHighlightedAlertId: (alertId) =>
    set({
      highlightedAlertId: alertId,
    }),
}));
