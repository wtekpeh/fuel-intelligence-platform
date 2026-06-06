import { CircleMarker, Popup } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";

function ReplayMarkerLayer() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);

  const replayHistoryReadings = useMapReplayStore(
    (state) => state.replayReadings,
  );

  if (!isReplayMode || !selectedDevice) {
    return null;
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
    return null;
  }

  return (
    <CircleMarker
      center={[replayReading.latitude, replayReading.longitude]}
      radius={14}
      pathOptions={{
        color: "#f8fafc",
        fillColor: "#f59e0b",
        fillOpacity: 0.95,
        opacity: 1,
        weight: 4,
      }}
    >
      <Popup>
        <strong>Replay Position</strong>
        <br />
        {new Date(replayReading.recorded_at).toLocaleString()}
        <br />
        Fuel: {replayReading.fuel_level_litres.toFixed(2)}L
      </Popup>
    </CircleMarker>
  );
}

export default ReplayMarkerLayer;
