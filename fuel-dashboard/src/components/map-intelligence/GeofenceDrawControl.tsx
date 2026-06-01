import { useEffect } from "react";
import { useMap } from "react-leaflet";

import L from "leaflet";
import "leaflet-draw";

import "leaflet-draw/dist/leaflet.draw.css";

import { useGeofenceDrawStore } from "../../store/geofenceDrawStore";
import { useMapReplayStore } from "../../store/mapReplayStore";

function GeofenceDrawControl() {
  const map = useMap();

  const startDrawing = useGeofenceDrawStore((state) => state.startDrawing);

  const setDrawnGeojson = useGeofenceDrawStore(
    (state) => state.setDrawnGeojson,
  );

  const pauseReplay = useMapReplayStore((state) => state.pause);

  useEffect(() => {
    const drawnItems = new L.FeatureGroup();

    map.addLayer(drawnItems);

    const drawControl = new L.Control.Draw({
      draw: {
        polygon: {
          allowIntersection: false,
          showArea: false,
          repeatMode: false,
          guidelineDistance: 12,
        },
        rectangle: {},
        circle: false,
        circlemarker: false,
        marker: false,
        polyline: false,
      },
      edit: {
        featureGroup: drawnItems,
        edit: false,
        remove: true,
      },
    });

    function handleDrawStart() {
      pauseReplay();

      startDrawing();
    }

    map.on(L.Draw.Event.DRAWSTART, handleDrawStart);
    map.addControl(drawControl);

    function handleCreated(event: L.LeafletEvent) {
      const createdEvent = event as L.DrawEvents.Created;

      drawnItems.clearLayers();

      drawnItems.addLayer(createdEvent.layer);

      const geojson = createdEvent.layer.toGeoJSON();

      console.log("DRAWN GEOFENCE GEOJSON:", geojson);

      if (geojson.geometry && geojson.geometry.type === "Polygon") {
        setDrawnGeojson({
          type: "Polygon",
          coordinates: geojson.geometry.coordinates,
        });
      }
    }

    map.on(L.Draw.Event.CREATED, handleCreated);

    return () => {
      map.off(L.Draw.Event.DRAWSTART, handleDrawStart);
      map.off(L.Draw.Event.CREATED, handleCreated);
      map.removeControl(drawControl);
      map.removeLayer(drawnItems);
    };
  }, [map, pauseReplay, setDrawnGeojson, startDrawing]);

  return null;
}

export default GeofenceDrawControl;
