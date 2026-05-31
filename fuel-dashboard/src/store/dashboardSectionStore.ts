import { create } from "zustand";

export type DashboardSection =
  | "operations"
  | "device-health"
  | "investigation"
  | "map"
  | "analytics";

interface DashboardSectionStore {
  activeSection: DashboardSection;
  setActiveSection: (section: DashboardSection) => void;
}

export const useDashboardSectionStore = create<DashboardSectionStore>(
  (set) => ({
    activeSection: "operations",

    setActiveSection: (section) =>
      set({
        activeSection: section,
      }),
  }),
);
