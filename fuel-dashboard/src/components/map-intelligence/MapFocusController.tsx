import { useEffect } from "react";
import { useMap } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useTelemetryStore } from "../../store/telemetryStore";

function MapFocusController() {
  const map = useMap();

  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  useEffect(() => {
    if (!selectedDevice) {
      return;
    }

    const selectedDeviceReading = readings.find(
      (reading) => reading.device_id === selectedDevice.device_id,
    );

    if (
      !selectedDeviceReading ||
      selectedDeviceReading.latitude === null ||
      selectedDeviceReading.longitude === null
    ) {
      return;
    }

    map.flyTo(
      [selectedDeviceReading.latitude, selectedDeviceReading.longitude],
      12,
      {
        duration: 1.2,
      },
    );
  }, [map, readings, selectedDevice]);

  return null;
}

export default MapFocusController;
