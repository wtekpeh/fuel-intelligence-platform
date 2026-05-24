import { httpClient } from "./httpClient";
import type { DeviceHealthEvent } from "../types";

export async function fetchDeviceHealthEvents(): Promise<DeviceHealthEvent[]> {
  const response = await httpClient.get<DeviceHealthEvent[]>(
    "/api/device-health-events",
  );

  return response.data;
}
