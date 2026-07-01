export interface AssetSummary {
  assetId: string;
  organizationId: string;
  name: string;
  assetType: string;
  metadata?: Record<string, unknown> | null;
  isActive: boolean;
  createdAt?: string;
}

export interface CreateAssetRequest {
  organizationId: string;
  name: string;
  assetType: string;
  metadata?: Record<string, unknown> | null;
}

export interface UpdateAssetRequest {
  name: string;
  assetType: string;
  metadata?: Record<string, unknown> | null;
}

export interface AssetMutationResponse {
  assetId: string;
  message: string;
}
