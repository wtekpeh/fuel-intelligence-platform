import { create } from "zustand";

interface MapReplayStore {
  isReplayMode: boolean;
  isPlaying: boolean;
  currentIndex: number;
  playbackSpeedMs: number;

  startReplay: () => void;
  stopReplay: () => void;
  play: () => void;
  pause: () => void;
  setCurrentIndex: (index: number) => void;
  setPlaybackSpeedMs: (speed: number) => void;
}

export const useMapReplayStore = create<MapReplayStore>((set) => ({
  isReplayMode: false,
  isPlaying: false,
  currentIndex: 0,
  playbackSpeedMs: 1000,

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
}));
