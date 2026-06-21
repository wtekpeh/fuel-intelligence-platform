import { httpClient } from "./httpClient";
import type {
  AlertTrendsResponse,
  GeofenceActivityTrendResponse,
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
