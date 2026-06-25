import { httpClient } from "../../api/httpClient";
import type {
  DeviceSensorSummary,
  DeviceSummary,
  HardwareProfile,
  HardwareProfileSensor,
  RegisterDeviceRequest,
} from "../types";

export const fetchHardwareProfiles = async (): Promise<HardwareProfile[]> => {
  const response = await httpClient.get<HardwareProfile[]>(
    "/api/hardware-profiles",
  );
  return response.data;
};

export const fetchHardwareProfileSensors = async (
  hardwareProfileId: string,
): Promise<HardwareProfileSensor[]> => {
  const response = await httpClient.get<HardwareProfileSensor[]>(
    `/api/hardware-profiles/${hardwareProfileId}/sensors`,
  );

  return response.data;
};

export const fetchDevices = async (): Promise<DeviceSummary[]> => {
  const response = await httpClient.get<DeviceSummary[]>("/api/devices");
  return response.data;
};

export const fetchDeviceSensors = async (
  deviceId: string,
): Promise<DeviceSensorSummary[]> => {
  const response = await httpClient.get<DeviceSensorSummary[]>(
    `/api/devices/${deviceId}/sensors`,
  );

  return response.data;
};

export const registerDevice = async (
  payload: RegisterDeviceRequest,
): Promise<string> => {
  const response = await httpClient.post<string>("/api/devices", payload);
  return response.data;
};
