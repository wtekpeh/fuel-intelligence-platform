import { httpClient } from "./httpClient";
import type { AlertLifecycleResponse, AlertResponse } from "../types";

export async function fetchAlerts(deviceId?: string): Promise<AlertResponse[]> {
  const response = await httpClient.get<AlertResponse[]>("/api/alerts", {
    params: {
      device_id: deviceId,
    },
  });

  return response.data;
}

export async function acknowledgeAlert(
  alertId: string,
): Promise<AlertLifecycleResponse> {
  const response = await httpClient.patch<AlertLifecycleResponse>(
    `/api/alerts/${alertId}/acknowledge`,
  );

  return response.data;
}

export async function resolveAlert(
  alertId: string,
): Promise<AlertLifecycleResponse> {
  const response = await httpClient.patch<AlertLifecycleResponse>(
    `/api/alerts/${alertId}/resolve`,
  );

  return response.data;
}
