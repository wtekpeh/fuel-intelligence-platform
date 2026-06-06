import { useEffect } from "react";
import { useMap } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import { useGeofenceDrawStore } from "../../store/geofenceDrawStore";

function ReplayCameraController() {
  const map = useMap();

  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);
  const replayHistoryReadings = useMapReplayStore(
    (state) => state.replayReadings,
  );
  const isDrawing = useGeofenceDrawStore((state) => state.isDrawing);

  useEffect(() => {
    if (!isReplayMode || !selectedDevice || isDrawing) {
      return;
    }

    const sourceReadings =
      replayHistoryReadings.length > 0 ? replayHistoryReadings : readings;

    const replayReadings = sourceReadings
      .filter(
        (reading) =>
          reading.device_id === selectedDevice.device_id &&
          reading.latitude !== null &&
          reading.longitude !== null,
      )
      .sort(
        (a, b) =>
          new Date(a.recorded_at).getTime() - new Date(b.recorded_at).getTime(),
      );

    const replayReading = replayReadings[currentIndex];

    if (
      !replayReading ||
      replayReading.latitude === null ||
      replayReading.longitude === null
    ) {
      return;
    }

    map.panTo([replayReading.latitude, replayReading.longitude], {
      animate: true,
      duration: 0.45,
    });
  }, [
    currentIndex,
    isReplayMode,
    map,
    readings,
    replayHistoryReadings,
    selectedDevice,
    isDrawing,
  ]);

  return null;
}

export default ReplayCameraController;
