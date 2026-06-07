import { create } from "zustand";

interface MapLayerStore {
  showDeviceMarker: boolean;
  showInvestigationEvents: boolean;
  showFuelEvents: boolean;
  showGeofenceTransitions: boolean;
  showGeofences: boolean;
  showReplayRoute: boolean;
  showHotspots: boolean;

  toggleDeviceMarker: () => void;
  toggleInvestigationEvents: () => void;
  toggleFuelEvents: () => void;
  toggleGeofenceTransitions: () => void;
  toggleGeofences: () => void;
  toggleReplayRoute: () => void;
  toggleHotspots: () => void;
}

export const useMapLayerStore = create<MapLayerStore>((set) => ({
  showDeviceMarker: true,
  showInvestigationEvents: true,
  showFuelEvents: true,
  showGeofenceTransitions: true,
  showGeofences: true,
  showReplayRoute: true,
  showHotspots: true,

  toggleDeviceMarker: () =>
    set((state) => ({
      showDeviceMarker: !state.showDeviceMarker,
    })),

  toggleInvestigationEvents: () =>
    set((state) => ({
      showInvestigationEvents: !state.showInvestigationEvents,
    })),

  toggleFuelEvents: () =>
    set((state) => ({
      showFuelEvents: !state.showFuelEvents,
    })),

  toggleGeofenceTransitions: () =>
    set((state) => ({
      showGeofenceTransitions: !state.showGeofenceTransitions,
    })),

  toggleGeofences: () =>
    set((state) => ({
      showGeofences: !state.showGeofences,
    })),

  toggleReplayRoute: () =>
    set((state) => ({
      showReplayRoute: !state.showReplayRoute,
    })),

  toggleHotspots: () =>
    set((state) => ({
      showHotspots: !state.showHotspots,
    })),
}));
