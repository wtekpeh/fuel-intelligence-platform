import { httpClient } from "./httpClient";

import type { DeviceStateEvent, FuelEvent, SensorHealthEvent } from "../types";

export async function fetchFuelEvents(deviceId?: string): Promise<FuelEvent[]> {
  const response = await httpClient.get<FuelEvent[]>("/api/fuel-events", {
    params: {
      device_id: deviceId,
    },
  });

  return response.data;
}

export async function fetchDeviceStateEvents(
  deviceId?: string,
): Promise<DeviceStateEvent[]> {
  const response = await httpClient.get<DeviceStateEvent[]>(
    "/api/device-state-events",
    {
      params: {
        device_id: deviceId,
      },
    },
  );

  return response.data;
}

export async function fetchSensorHealthEvents(
  deviceId?: string,
): Promise<SensorHealthEvent[]> {
  const response = await httpClient.get<SensorHealthEvent[]>(
    "/api/sensor-health-events",
    {
      params: {
        device_id: deviceId,
      },
    },
  );

  return response.data;
}
