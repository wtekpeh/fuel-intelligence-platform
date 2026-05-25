import { create } from "zustand";

export type AppView = "landing" | "fleet" | "dashboard";

interface AppViewStore {
  activeView: AppView;

  showLanding: () => void;
  showFleet: () => void;
  showDashboard: () => void;
}

export const useAppViewStore = create<AppViewStore>((set) => ({
  activeView: "landing",

  showLanding: () =>
    set({
      activeView: "landing",
    }),

  showFleet: () =>
    set({
      activeView: "fleet",
    }),

  showDashboard: () =>
    set({
      activeView: "dashboard",
    }),
}));
