import { httpClient } from "./httpClient";

import type { CreateGeofencePayload, Geofence } from "../types";

export async function listGeofences(
  organizationId: string,
): Promise<Geofence[]> {
  const response = await httpClient.get<Geofence[]>(
    `/api/geofences/${organizationId}`,
  );

  return response.data;
}

export async function createGeofence(
  payload: CreateGeofencePayload,
): Promise<Geofence> {
  const response = await httpClient.post<Geofence>("/api/geofences", payload);

  return response.data;
}
