import OperationalMap from "./OperationalMap";
import FuelLevelGauge from "../shared/FuelLevelGauge";

import { useFleetStore } from "../../store/fleetStore";
import { useTelemetryStore } from "../../store/telemetryStore";

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
            </>
          ) : (
            <div className="map-live-card">
              <label>Telemetry</label>
              <strong>No live telemetry available</strong>
            </div>
          )}
        </aside>
      </div>
    </section>
  );
}

export default MapIntelligencePanel;
