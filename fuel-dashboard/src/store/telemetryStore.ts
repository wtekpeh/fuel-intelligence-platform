import { create } from "zustand";

import type { TelemetryStreamReading } from "../types";

interface TelemetryStore {
  readings: TelemetryStreamReading[];

  setReadings: (readings: TelemetryStreamReading[]) => void;
}

export const useTelemetryStore = create<TelemetryStore>((set) => ({
  readings: [],

  setReadings: (readings) =>
    set({
      readings,
    }),
}));
