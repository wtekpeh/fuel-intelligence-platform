import { httpClient } from "./httpClient";

import type {
  CheckPositionPayload,
  CheckPositionResponse,
  GeofenceTransitionEvent,
  CreateGeofencePayload,
  Geofence,
} from "../types";

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

export async function checkPositionAgainstGeofences(
  payload: CheckPositionPayload,
): Promise<CheckPositionResponse> {
  const response = await httpClient.post<CheckPositionResponse>(
    "/api/geofences/check-position",
    payload,
  );

  return response.data;
}

export async function listGeofenceTransitionEvents(
  deviceId?: string,
): Promise<GeofenceTransitionEvent[]> {
  const response = await httpClient.get<GeofenceTransitionEvent[]>(
    "/api/geofence-transition-events",
    {
      params: {
        device_id: deviceId,
      },
    },
  );

  return response.data;
}
