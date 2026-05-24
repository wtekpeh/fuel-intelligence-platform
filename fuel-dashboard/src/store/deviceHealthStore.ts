import { create } from "zustand";

import type { DeviceHealthEvent, DeviceHealthStatus } from "../types";

interface DeviceHealthStore {
  events: DeviceHealthEvent[];

  setEvents: (events: DeviceHealthEvent[]) => void;

  onlineCount: number;
  staleCount: number;
  offlineCount: number;
  unknownCount: number;
}

function countLatestDeviceStatuses(events: DeviceHealthEvent[]) {
  const latestStatusByDevice = new Map<string, DeviceHealthStatus>();

  for (const event of events) {
    if (!latestStatusByDevice.has(event.device_id)) {
      latestStatusByDevice.set(event.device_id, event.new_status);
    }
  }

  return {
    onlineCount: Array.from(latestStatusByDevice.values()).filter(
      (status) => status === "ONLINE",
    ).length,

    staleCount: Array.from(latestStatusByDevice.values()).filter(
      (status) => status === "STALE",
    ).length,

    offlineCount: Array.from(latestStatusByDevice.values()).filter(
      (status) => status === "OFFLINE",
    ).length,

    unknownCount: Array.from(latestStatusByDevice.values()).filter(
      (status) => status === "UNKNOWN",
    ).length,
  };
}

export const useDeviceHealthStore = create<DeviceHealthStore>((set) => ({
  events: [],

  onlineCount: 0,
  staleCount: 0,
  offlineCount: 0,
  unknownCount: 0,

  setEvents: (events) => {
    const counts = countLatestDeviceStatuses(events);

    set({
      events,
      ...counts,
    });
  },
}));
