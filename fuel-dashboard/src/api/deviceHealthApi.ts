import { httpClient } from "./httpClient";
import type { DeviceHealthEvent } from "../types";

export async function fetchDeviceHealthEvents(
  deviceId?: string,
): Promise<DeviceHealthEvent[]> {
  const response = await httpClient.get<DeviceHealthEvent[]>(
    "/api/device-health-events",
    {
      params: {
        device_id: deviceId,
      },
    },
  );

  return response.data;
}
