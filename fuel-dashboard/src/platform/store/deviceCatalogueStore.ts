import { create } from "zustand";
import { fetchDeviceCatalogue } from "../../api/platformApi";
import type { DeviceCatalogueModel } from "../../types/hardware";

interface DeviceCatalogueStore {
  models: DeviceCatalogueModel[];
  selectedModel: DeviceCatalogueModel | null;
  loading: boolean;
  error: string | null;

  loadCatalogue: () => Promise<void>;
  selectModel: (model: DeviceCatalogueModel | null) => void;
}

export const useDeviceCatalogueStore = create<DeviceCatalogueStore>((set) => ({
  models: [],
  selectedModel: null,
  loading: false,
  error: null,

  loadCatalogue: async () => {
    set({ loading: true, error: null });

    try {
      const models = await fetchDeviceCatalogue();

      set({
        models,
        selectedModel: models[0] ?? null,
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to load device catalogue.",
      });
    }
  },

  selectModel: (model) => {
    set({ selectedModel: model });
  },
}));
