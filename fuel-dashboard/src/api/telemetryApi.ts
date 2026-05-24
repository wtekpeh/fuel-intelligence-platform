import { httpClient } from "./httpClient";
import type { TelemetryStreamReading } from "../types";

export async function fetchRecentTelemetry(): Promise<
  TelemetryStreamReading[]
> {
  const response = await httpClient.get<TelemetryStreamReading[]>(
    "/api/fuel-readings/recent",
  );

  return response.data;
}
