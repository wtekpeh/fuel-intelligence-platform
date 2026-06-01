import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";

function ReplayStatusCard() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const isPlaying = useMapReplayStore((state) => state.isPlaying);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);
  const setCurrentIndex = useMapReplayStore((state) => state.setCurrentIndex);

  if (!isReplayMode || !selectedDevice) {
    return null;
  }

  const replayReadings = readings
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

  const currentReading = replayReadings[currentIndex];

  if (!currentReading) {
    return (
      <div className="map-live-card">
        <label>Replay Status</label>
        <strong>No replay telemetry available</strong>
      </div>
    );
  }

  return (
    <div className="map-live-card">
      <label>Replay Status</label>

      <strong>{isPlaying ? "Playing" : "Paused"}</strong>

      <span>
        Point {currentIndex + 1} of {replayReadings.length}
      </span>

      <span>{new Date(currentReading.recorded_at).toLocaleString()}</span>

      <span>Fuel: {currentReading.fuel_level_litres.toFixed(2)}L</span>

      <input
        type="range"
        min={0}
        max={Math.max(replayReadings.length - 1, 0)}
        value={currentIndex}
        onChange={(event) => setCurrentIndex(Number(event.target.value))}
        className="replay-status-card__slider"
      />
    </div>
  );
}

export default ReplayStatusCard;
