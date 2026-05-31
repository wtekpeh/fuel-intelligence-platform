import { Marker, Popup } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useTelemetryStore } from "../../store/telemetryStore";

function DeviceMarkerLayer() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const readings = useTelemetryStore((state) => state.readings);

  if (!selectedDevice) {
    return null;
  }

  const selectedDeviceReading = readings.find(
    (reading) => reading.device_id === selectedDevice.device_id,
  );

  if (
    !selectedDeviceReading ||
    selectedDeviceReading.latitude === null ||
    selectedDeviceReading.longitude === null
  ) {
    return null;
  }

  const position: [number, number] = [
    selectedDeviceReading.latitude,
    selectedDeviceReading.longitude,
  ];

  return (
    <Marker position={position}>
      <Popup>
        <div>
          <strong>{selectedDevice.device_code}</strong>

          <br />

          <span>{selectedDevice.asset_name}</span>

          <hr />

          <div>
            Fuel Level: {selectedDeviceReading.fuel_level_litres.toFixed(2)}L
          </div>

          <div>
            Motion:{" "}
            {selectedDeviceReading.motion_detected ? "Detected" : "Idle"}
          </div>

          <div>Vibration: {selectedDeviceReading.vibration_level ?? "N/A"}</div>

          <div>
            Telemetry Time:{" "}
            {new Date(selectedDeviceReading.recorded_at).toLocaleString()}
          </div>
        </div>
      </Popup>
    </Marker>
  );
}

export default DeviceMarkerLayer;
