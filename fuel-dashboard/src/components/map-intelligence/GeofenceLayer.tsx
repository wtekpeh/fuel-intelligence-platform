import { GeoJSON } from "react-leaflet";

import type { GeoJSON as GeoJSONType } from "geojson";

import { useGeofenceStore } from "../../store/geofenceStore";

import type { Geofence } from "../../types";

function getGeofenceColor(geofenceType: string) {
  switch (geofenceType) {
    case "DEPOT":
      return "#22c55e";

    case "FUELING_STATION":
      return "#3b82f6";

    case "RESTRICTED_ZONE":
      return "#ef4444";

    case "SAFE_CORRIDOR":
      return "#f59e0b";

    case "CUSTOMER_SITE":
      return "#8b5cf6";

    default:
      return "#38bdf8";
  }
}

function GeofenceLayer() {
  const geofences = useGeofenceStore((state) => state.geofences);

  return (
    <>
      {geofences.map((geofence: Geofence) => {
        const color = getGeofenceColor(geofence.geofence_type);

        return (
          <GeoJSON
            key={geofence.id}
            data={geofence.geojson as GeoJSONType}
            style={{
              color,
              fillColor: color,
              fillOpacity: 0.18,
              opacity: 1,
              weight: 3,
            }}
          />
        );
      })}
    </>
  );
}

export default GeofenceLayer;
