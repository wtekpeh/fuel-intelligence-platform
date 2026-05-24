import { create } from "zustand";

export type ConnectionStatus =
  | "connected"
  | "connecting"
  | "disconnected"
  | "error";

interface ConnectionStore {
  status: ConnectionStatus;
  lastHeartbeatAt: string | null;

  setStatus: (status: ConnectionStatus) => void;
  setLastHeartbeatAt: (timestamp: string) => void;
}

export const useConnectionStore = create<ConnectionStore>((set) => ({
  status: "disconnected",

  lastHeartbeatAt: null,

  setStatus: (status) =>
    set({
      status,
    }),

  setLastHeartbeatAt: (timestamp) =>
    set({
      lastHeartbeatAt: timestamp,
    }),
}));
