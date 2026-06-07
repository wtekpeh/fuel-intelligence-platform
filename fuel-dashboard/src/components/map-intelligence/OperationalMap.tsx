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
import FuelEventHotspotLayer from "./FuelEventHotspotLayer";
import { useMapLayerStore } from "../../store/mapLayerStore";
import HotspotClusterLayer from "./HotspotClusterLayer";

function OperationalMap() {
  const showFuelEvents = useMapLayerStore((state) => state.showFuelEvents);

  const showDeviceMarker = useMapLayerStore((state) => state.showDeviceMarker);

  const showInvestigationEvents = useMapLayerStore(
    (state) => state.showInvestigationEvents,
  );

  const showGeofenceTransitions = useMapLayerStore(
    (state) => state.showGeofenceTransitions,
  );

  const showGeofences = useMapLayerStore((state) => state.showGeofences);

  const showReplayRoute = useMapLayerStore((state) => state.showReplayRoute);

  const showHotspots = useMapLayerStore((state) => state.showHotspots);

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

        {showGeofences && <GeofenceLayer />}

        {showReplayRoute && <TelemetryRouteLayer />}

        {showFuelEvents && <FuelEventHotspotLayer />}

        {showHotspots && <HotspotClusterLayer />}

        <ReplayMarkerLayer />

        {showDeviceMarker && <DeviceMarkerLayer />}

        {showInvestigationEvents && <InvestigationEventLayer />}

        {showGeofenceTransitions && <GeofenceTransitionLayer />}
        <MapFocusController />
        <ReplayCameraController />
        <ReplayPlaybackController />
        <GeofenceDrawControl />
      </MapContainer>
    </div>
  );
}

export default OperationalMap;
