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
