export type GeofenceType =
  | "DEPOT"
  | "FUELING_STATION"
  | "RESTRICTED_ZONE"
  | "SAFE_CORRIDOR"
  | "CUSTOMER_SITE";

export interface GeoJsonPolygon {
  type: "Polygon";
  coordinates: number[][][];
}

export interface Geofence {
  id: string;
  organization_id: string;
  name: string;
  geofence_type: GeofenceType | string;
  geojson: GeoJsonPolygon;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateGeofencePayload {
  organization_id: string;
  name: string;
  geofence_type: GeofenceType;
  geojson: GeoJsonPolygon;
}

export interface CheckPositionPayload {
  organization_id: string;
  device_id: string;
  latitude: number;
  longitude: number;
}

export interface GeofencePositionMatch {
  geofence_id: string;
  geofence_name: string;
  geofence_type: string;
}

export interface CheckPositionResponse {
  inside_geofence: boolean;
  matched_geofences: GeofencePositionMatch[];
}

export interface GeofenceTransitionEvent {
  id: string;
  organization_id: string;
  device_id: string;

  geofence_id: string;
  geofence_name: string;
  geofence_type: string;

  transition_type: "ENTERED_ZONE" | "EXITED_ZONE" | string;

  latitude: number;
  longitude: number;

  recorded_at: string;
  detected_at: string;
  created_at: string;
}
