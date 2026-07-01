export interface DeviceSummary {
  id: string;
  deviceCode: string;
  assetId: string;
  deviceModelId?: string | null;
  hardwareProfileId: string;

  hardwareProfileCode: string;
  hardwareProfileName: string;

  deviceModelCode?: string | null;
  deviceModelName?: string | null;

  status: string;
  isActive: boolean;

  createdAt: string;
  lastSeenAt?: string | null;
}

export interface RegisterDeviceRequest {
  assetId: string;
  device_model_id?: string | null;
  deviceModelId?: string | null;
  hardwareProfileId: string;
  deviceCode: string;
}

export interface UpdateDeviceRequest {
  hardwareProfileId: string;
}

export interface AssignDeviceAssetRequest {
  assetId: string;
}

export interface DeviceMutationResponse {
  deviceId: string;
  message: string;
}

export interface ProvisionedSensorResponse {
  id: string;
  sensorCode: string;
  sensorType: string;
  unit: string;
}
