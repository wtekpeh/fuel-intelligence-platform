import { create } from "zustand";

import type { GeoJsonPolygon } from "../types";

interface GeofenceDrawStore {
  isDrawing: boolean;
  drawnGeojson: GeoJsonPolygon | null;

  startDrawing: () => void;
  stopDrawing: () => void;
  setDrawnGeojson: (geojson: GeoJsonPolygon) => void;
  clearDrawnGeojson: () => void;
}

export const useGeofenceDrawStore = create<GeofenceDrawStore>((set) => ({
  isDrawing: false,
  drawnGeojson: null,

  startDrawing: () =>
    set({
      isDrawing: true,
      drawnGeojson: null,
    }),

  stopDrawing: () =>
    set({
      isDrawing: false,
    }),

  setDrawnGeojson: (geojson) =>
    set({
      drawnGeojson: geojson,
      isDrawing: false,
    }),

  clearDrawnGeojson: () =>
    set({
      drawnGeojson: null,
      isDrawing: false,
    }),
}));
