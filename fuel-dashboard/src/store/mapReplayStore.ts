import { create } from "zustand";
import type { TelemetryStreamReading } from "../types";

interface MapReplayStore {
  isReplayMode: boolean;
  isPlaying: boolean;
  currentIndex: number;
  playbackSpeedMs: number;

  replayReadings: TelemetryStreamReading[];

  selectedStartTime: string | null;
  selectedEndTime: string | null;

  startReplay: () => void;
  stopReplay: () => void;
  play: () => void;
  pause: () => void;
  setCurrentIndex: (index: number) => void;
  setPlaybackSpeedMs: (speed: number) => void;

  setReplayReadings: (readings: TelemetryStreamReading[]) => void;

  setReplayTimeRange: (
    startTime: string | null,
    endTime: string | null,
  ) => void;
}

export const useMapReplayStore = create<MapReplayStore>((set) => ({
  isReplayMode: false,
  isPlaying: false,
  currentIndex: 0,
  playbackSpeedMs: 1000,

  replayReadings: [],

  selectedStartTime: null,
  selectedEndTime: null,

  startReplay: () =>
    set({
      isReplayMode: true,
      isPlaying: false,
      currentIndex: 0,
    }),

  stopReplay: () =>
    set({
      isReplayMode: false,
      isPlaying: false,
      currentIndex: 0,
    }),

  play: () =>
    set({
      isPlaying: true,
    }),

  pause: () =>
    set({
      isPlaying: false,
    }),

  setCurrentIndex: (index) =>
    set({
      currentIndex: index,
    }),

  setPlaybackSpeedMs: (speed) =>
    set({
      playbackSpeedMs: speed,
    }),

  setReplayReadings: (readings) =>
    set({
      replayReadings: readings,
      currentIndex: 0,
    }),

  setReplayTimeRange: (startTime, endTime) =>
    set({
      selectedStartTime: startTime,
      selectedEndTime: endTime,
    }),
}));
