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
