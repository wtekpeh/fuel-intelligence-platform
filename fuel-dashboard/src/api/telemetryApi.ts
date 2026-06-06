import { httpClient } from "./httpClient";
import type { TelemetryStreamReading } from "../types";

export async function fetchRecentTelemetry(
  deviceId?: string,
): Promise<TelemetryStreamReading[]> {
  const response = await httpClient.get<TelemetryStreamReading[]>(
    "/api/fuel-readings/recent",
    {
      params: {
        device_id: deviceId,
      },
    },
  );

  return response.data;
}

export async function fetchTelemetryHistory(
  deviceId: string,
  startTime: string,
  endTime: string,
): Promise<TelemetryStreamReading[]> {
  const response = await httpClient.get<TelemetryStreamReading[]>(
    "/api/fuel-readings/history",
    {
      params: {
        device_id: deviceId,
        start_time: startTime,
        end_time: endTime,
      },
    },
  );

  return response.data;
}
