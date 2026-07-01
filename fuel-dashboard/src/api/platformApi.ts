import { httpClient } from "./httpClient";

import type {
  AssetMutationResponse,
  CreateAssetRequest,
  UpdateAssetRequest,
  DeviceModelResponse,
  DeviceMutationResponse,
  DeviceSummary,
  HardwareProfileResponse,
  HardwareProfileSensorResponse,
  RegisterDeviceRequest,
  UpdateDeviceRequest,
  AssignDeviceAssetRequest,
  DeviceCatalogueModel,
} from "../types";

interface HardwareProfileApiResponse {
  id: string;
  profile_code: string;
  name: string;
  description: string | null;
  is_active: boolean;
  created_at: string;
}

interface HardwareProfileSensorApiResponse {
  id: string;
  hardware_profile_id: string;
  sensor_type: string;
  unit: string;
}

interface DeviceModelApiResponse {
  id: string;
  model_code: string;
  model_name: string;
  manufacturer: string;
  description: string | null;
  is_active: boolean;
  created_at: string;
}

interface DeviceCatalogueSensorApiResponse {
  id: string;
  sensor_type: string;
  unit: string;
}

interface DeviceCatalogueHardwareProfileApiResponse {
  id: string;
  profile_code: string;
  name: string;
  description: string | null;
  is_default: boolean;
  sensors: DeviceCatalogueSensorApiResponse[];
}

interface DeviceCatalogueModelApiResponse {
  id: string;
  model_code: string;
  model_name: string;
  manufacturer: string | null;
  description: string | null;
  is_active: boolean;
  profiles: DeviceCatalogueHardwareProfileApiResponse[];
}

export async function createAsset(
  request: CreateAssetRequest,
): Promise<AssetMutationResponse> {
  const response = await httpClient.post<AssetMutationResponse>(
    "/api/assets",
    request,
  );

  return response.data;
}

export async function updateAsset(
  assetId: string,
  request: UpdateAssetRequest,
): Promise<AssetMutationResponse> {
  const response = await httpClient.patch<AssetMutationResponse>(
    `/api/assets/${assetId}`,
    request,
  );

  return response.data;
}

export async function deleteAsset(
  assetId: string,
): Promise<AssetMutationResponse> {
  const response = await httpClient.delete<AssetMutationResponse>(
    `/api/assets/${assetId}`,
  );

  return response.data;
}

export async function registerDevice(
  request: RegisterDeviceRequest,
): Promise<string> {
  const response = await httpClient.post<string>("/api/devices", request);

  return response.data;
}

export async function fetchDevices(): Promise<DeviceSummary[]> {
  const response = await httpClient.get<DeviceSummary[]>("/api/devices");

  return response.data;
}

export async function updateDevice(
  deviceId: string,
  request: UpdateDeviceRequest,
): Promise<DeviceMutationResponse> {
  const response = await httpClient.patch<DeviceMutationResponse>(
    `/api/devices/${deviceId}`,
    request,
  );

  return response.data;
}

export async function assignDeviceToAsset(
  deviceId: string,
  request: AssignDeviceAssetRequest,
): Promise<DeviceMutationResponse> {
  const response = await httpClient.patch<DeviceMutationResponse>(
    `/api/devices/${deviceId}/assign-asset`,
    request,
  );

  return response.data;
}

export async function deactivateDevice(
  deviceId: string,
): Promise<DeviceMutationResponse> {
  const response = await httpClient.delete<DeviceMutationResponse>(
    `/api/devices/${deviceId}`,
  );

  return response.data;
}

export async function fetchHardwareProfiles(): Promise<
  HardwareProfileResponse[]
> {
  const response = await httpClient.get<HardwareProfileApiResponse[]>(
    "/api/hardware-profiles",
  );

  return response.data.map((profile) => ({
    id: profile.id,
    profileCode: profile.profile_code,
    name: profile.name,
    description: profile.description,
    isActive: profile.is_active,
    createdAt: profile.created_at,
  }));
}

export async function fetchHardwareProfileSensors(
  hardwareProfileId: string,
): Promise<HardwareProfileSensorResponse[]> {
  const response = await httpClient.get<HardwareProfileSensorApiResponse[]>(
    `/api/hardware-profiles/${hardwareProfileId}/sensors`,
  );

  return response.data.map((sensor) => ({
    id: sensor.id,
    hardwareProfileId: sensor.hardware_profile_id,
    sensorType: sensor.sensor_type,
    unit: sensor.unit,
  }));
}

export async function fetchDeviceModels(): Promise<DeviceModelResponse[]> {
  const response =
    await httpClient.get<DeviceModelApiResponse[]>("/api/device-models");

  return response.data.map((model) => ({
    id: model.id,
    modelCode: model.model_code,
    modelName: model.model_name,
    manufacturer: model.manufacturer,
    description: model.description,
    isActive: model.is_active,
    createdAt: model.created_at,
  }));
}

export async function fetchDeviceCatalogue(): Promise<DeviceCatalogueModel[]> {
  const response = await httpClient.get<DeviceCatalogueModelApiResponse[]>(
    "/api/device-catalogue",
  );

  return response.data.map((model) => ({
    id: model.id,
    modelCode: model.model_code,
    modelName: model.model_name,
    manufacturer: model.manufacturer,
    description: model.description,
    isActive: model.is_active,
    profiles: model.profiles.map((profile) => ({
      id: profile.id,
      profileCode: profile.profile_code,
      name: profile.name,
      description: profile.description,
      isDefault: profile.is_default,
      sensors: profile.sensors.map((sensor) => ({
        id: sensor.id,
        sensorType: sensor.sensor_type,
        unit: sensor.unit,
      })),
    })),
  }));
}
