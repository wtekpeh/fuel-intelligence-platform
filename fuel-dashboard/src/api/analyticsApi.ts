import { httpClient } from "./httpClient";
import type {
  AlertTrendsResponse,
  GeofenceActivityTrendResponse,
  DeviceHealthTrendResponse,
  GeofenceUtilizationResponse,
} from "../types";

export const getAlertTrends = async (
  days: number = 30,
): Promise<AlertTrendsResponse> => {
  const response = await httpClient.get<AlertTrendsResponse>(
    `/api/analytics/alert-trends?days=${days}`,
  );

  return response.data;
};

export const getGeofenceActivityTrends = async (
  days: number = 30,
): Promise<GeofenceActivityTrendResponse> => {
  const response = await httpClient.get<GeofenceActivityTrendResponse>(
    `/api/analytics/geofence-activity?days=${days}`,
  );

  return response.data;
};

export async function getDeviceHealthTrends(
  days = 30,
): Promise<DeviceHealthTrendResponse> {
  const response = await httpClient.get<DeviceHealthTrendResponse>(
    `/api/analytics/device-health-trends?days=${days}`,
  );

  return response.data;
}

export async function getGeofenceUtilization(
  days = 30,
): Promise<GeofenceUtilizationResponse> {
  const response = await httpClient.get<GeofenceUtilizationResponse>(
    `/api/analytics/geofence-utilization?days=${days}`,
  );

  return response.data;
}
