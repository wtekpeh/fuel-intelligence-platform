import { create } from "zustand";

import { fetchOrganizationFleetOverview } from "../../api/fleetApi";

import type { OrganizationFleetOverview } from "../../types";

import { createAsset } from "../../api/platformApi";
import type { CreateAssetRequest } from "../../types";

export interface PlatformAssetSummary {
  asset_id: string;
  asset_name: string;
  asset_type: string;
  capacity_litres: number | null;
  device_count: number;
  sensor_count: number;
  open_alert_count: number;
  rows: OrganizationFleetOverview[];
}

interface AssetStore {
  assets: PlatformAssetSummary[];
  selectedAsset: PlatformAssetSummary | null;
  selectedAssetRows: OrganizationFleetOverview[];

  loading: boolean;
  error: string | null;

  loadAssets: (organizationId: string) => Promise<void>;

  createAsset: (request: CreateAssetRequest) => Promise<string | null>;

  selectAsset: (asset: PlatformAssetSummary | null) => void;

  addLocalAsset: (asset: PlatformAssetSummary) => void;

  clearAssets: () => void;
}

const groupFleetRowsIntoAssets = (
  rows: OrganizationFleetOverview[],
): PlatformAssetSummary[] => {
  const groupedAssets = new Map<string, PlatformAssetSummary>();

  rows.forEach((row) => {
    const existingAsset = groupedAssets.get(row.asset_id);

    if (existingAsset) {
      existingAsset.rows.push(row);

      if (row.device_id) {
        existingAsset.device_count += 1;
      }

      existingAsset.sensor_count += row.sensor_count ?? 0;
      existingAsset.open_alert_count += row.open_alert_count ?? 0;

      return;
    }

    groupedAssets.set(row.asset_id, {
      asset_id: row.asset_id,
      asset_name: row.asset_name,
      asset_type: row.asset_type,
      capacity_litres: row.capacity_litres,
      device_count: row.device_id ? 1 : 0,
      sensor_count: row.sensor_count ?? 0,
      open_alert_count: row.open_alert_count ?? 0,
      rows: [row],
    });
  });

  return Array.from(groupedAssets.values());
};

export const useAssetStore = create<AssetStore>((set) => ({
  assets: [],

  selectedAsset: null,
  selectedAssetRows: [],

  loading: false,

  error: null,

  loadAssets: async (organizationId: string) => {
    set({
      loading: true,
      error: null,
    });

    try {
      const fleetRows = await fetchOrganizationFleetOverview(organizationId);

      const assets = groupFleetRowsIntoAssets(fleetRows);
      const selectedAsset = assets[0] ?? null;

      set({
        assets,
        selectedAsset,
        selectedAssetRows: selectedAsset?.rows ?? [],
        loading: false,
      });
    } catch {
      set({
        loading: false,
        error: "Failed to load organization assets.",
      });
    }
  },

  createAsset: async (request) => {
    set({
      loading: true,
      error: null,
    });

    try {
      const result = await createAsset(request);

      set({
        loading: false,
      });

      return result.assetId;
    } catch {
      set({
        loading: false,
        error: "Failed to create asset.",
      });

      return null;
    }
  },

  addLocalAsset: (asset) => {
    set((state) => ({
      assets: [asset, ...state.assets],
      selectedAsset: asset,
      selectedAssetRows: asset.rows,
    }));
  },

  selectAsset: (asset) => {
    set({
      selectedAsset: asset,
      selectedAssetRows: asset?.rows ?? [],
    });
  },

  clearAssets: () => {
    set({
      assets: [],
      selectedAsset: null,
      selectedAssetRows: [],
    });
  },
}));
