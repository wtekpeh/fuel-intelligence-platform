import { MapContainer, TileLayer } from "react-leaflet";
import DeviceMarkerLayer from "./DeviceMarkerLayer";
import MapFocusController from "./MapFocusController";
import InvestigationEventLayer from "./InvestigationEventLayer";
import TelemetryRouteLayer from "./TelemetryRouteLayer";
import ReplayPlaybackController from "./ReplayPlaybackController";
import ReplayMarkerLayer from "./ReplayMarkerLayer";
import ReplayCameraController from "./ReplayCameraController";
import GeofenceLayer from "./GeofenceLayer";
import GeofenceDrawControl from "./GeofenceDrawControl";
import GeofenceTransitionLayer from "./GeofenceTransitionLayer";

function OperationalMap() {
  return (
    <div className="operational-map-shell">
      <MapContainer
        center={[5.6037, -0.187]}
        zoom={12}
        className="operational-map"
        scrollWheelZoom
      >
        <TileLayer
          attribution="&copy; OpenStreetMap contributors"
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />

        <GeofenceLayer />
        <TelemetryRouteLayer />
        <ReplayMarkerLayer />
        <DeviceMarkerLayer />
        <InvestigationEventLayer />
        <GeofenceTransitionLayer />
        <MapFocusController />
        <ReplayCameraController />
        <ReplayPlaybackController />
        <GeofenceDrawControl />
      </MapContainer>
    </div>
  );
}

export default OperationalMap;
