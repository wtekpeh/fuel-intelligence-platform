import { create } from "zustand";

import type { DeviceStateEvent, FuelEvent, SensorHealthEvent } from "../types";
import type { InvestigationTimelineItem } from "../utils/investigationTimeline";

interface InvestigationStore {
  fuelEvents: FuelEvent[];
  deviceStateEvents: DeviceStateEvent[];
  sensorHealthEvents: SensorHealthEvent[];
  selectedTimelineItem: InvestigationTimelineItem | null;
  focusedFuelEventId: string | null;

  setFuelEvents: (events: FuelEvent[]) => void;
  setDeviceStateEvents: (events: DeviceStateEvent[]) => void;
  setSensorHealthEvents: (events: SensorHealthEvent[]) => void;
  selectTimelineItem: (item: InvestigationTimelineItem) => void;

  clearSelectedTimelineItem: () => void;
  setFocusedFuelEventId: (fuelEventId: string | null) => void;
}

export const useInvestigationStore = create<InvestigationStore>((set) => ({
  fuelEvents: [],
  deviceStateEvents: [],
  sensorHealthEvents: [],
  selectedTimelineItem: null,
  focusedFuelEventId: null,

  setFuelEvents: (events) =>
    set({
      fuelEvents: events,
    }),

  setDeviceStateEvents: (events) =>
    set({
      deviceStateEvents: events,
    }),

  setSensorHealthEvents: (events) =>
    set({
      sensorHealthEvents: events,
    }),

  selectTimelineItem: (item) =>
    set({
      selectedTimelineItem: item,
    }),

  clearSelectedTimelineItem: () =>
    set({
      selectedTimelineItem: null,
    }),

  setFocusedFuelEventId: (fuelEventId) =>
    set({
      focusedFuelEventId: fuelEventId,
    }),
}));
