import { MapContainer, TileLayer } from "react-leaflet";
import DeviceMarkerLayer from "./DeviceMarkerLayer";
import MapFocusController from "./MapFocusController";
import InvestigationEventLayer from "./InvestigationEventLayer";

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

        <DeviceMarkerLayer />

        <InvestigationEventLayer />

        <MapFocusController />
      </MapContainer>
    </div>
  );
}

export default OperationalMap;
