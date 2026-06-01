import OperationalMap from "./OperationalMap";
import FuelLevelGauge from "../shared/FuelLevelGauge";
import ReplayControls from "./ReplayControls";
import ReplayStatusCard from "./ReplayStatusCard";
import GeofenceStatusCard from "./GeofenceStatusCard";

import { useFleetStore } from "../../store/fleetStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import GeofenceCreationCard from "./GeofenceCreationCard";

function MapIntelligencePanel() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const readings = useTelemetryStore((state) => state.readings);

  const selectedDeviceReading = readings.find(
    (reading) => reading.device_id === selectedDevice?.device_id,
  );

  return (
    <section className="map-intelligence-panel">
      <div className="map-intelligence-panel__header">
        <h2>Map Intelligence</h2>
        <p>
          Spatial operational view for selected device telemetry, investigation
          events, and future route replay intelligence.
        </p>
      </div>

      <div className="map-intelligence-workspace">
        <div className="map-intelligence-workspace__map">
          <OperationalMap />
        </div>

        <aside className="map-intelligence-workspace__side">
          <ReplayControls />

          <ReplayStatusCard />

          <GeofenceStatusCard />

          {selectedDeviceReading ? (
            <>
              <FuelLevelGauge
                value={selectedDeviceReading.fuel_level_litres}
                maxValue={selectedDevice?.capacity_litres ?? 100}
                size="large"
                label={`${selectedDevice?.device_code ?? "Device"} Fuel`}
              />

              <div className="map-live-card">
                <label>Latest Telemetry</label>

                <strong>
                  {new Date(selectedDeviceReading.recorded_at).toLocaleString()}
                </strong>
              </div>

              <div className="map-live-card">
                <label>Motion</label>

                <strong>
                  {selectedDeviceReading.motion_detected ? "Detected" : "Idle"}
                </strong>
              </div>

              <div className="map-live-card">
                <label>Vibration</label>

                <strong>
                  {selectedDeviceReading.vibration_level ?? "N/A"}
                </strong>
              </div>

              <div className="map-live-card">
                <label>Route Context</label>
                <strong>Recent Telemetry Path</strong>
                <span>
                  Path is reconstructed from the latest telemetry readings for
                  this selected device.
                </span>
              </div>
            </>
          ) : (
            <div className="map-live-card">
              <label>Telemetry</label>
              <strong>No live telemetry available</strong>
            </div>
          )}
        </aside>
      </div>

      <GeofenceCreationCard />
    </section>
  );
}

export default MapIntelligencePanel;
