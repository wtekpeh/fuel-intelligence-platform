import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import { useGeofenceStore } from "../../store/geofenceStore";
import { useInvestigationStore } from "../../store/investigationStore";
import { useAlertStore } from "../../store/alertStore";
import { useDeviceHealthStore } from "../../store/deviceHealthStore";

function ReplayStatusCard() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const isPlaying = useMapReplayStore((state) => state.isPlaying);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);
  const replayHistoryReadings = useMapReplayStore(
    (state) => state.replayReadings,
  );
  const transitionEvents = useGeofenceStore((state) => state.transitionEvents);
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);
  const deviceStateEvents = useInvestigationStore(
    (state) => state.deviceStateEvents,
  );
  const alerts = useAlertStore((state) => state.alerts);
  const deviceHealthEvents = useDeviceHealthStore((state) => state.events);
  const setCurrentIndex = useMapReplayStore((state) => state.setCurrentIndex);

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

  const currentReading = replayReadings[currentIndex];

  const currentTimestamp = currentReading
    ? new Date(currentReading.recorded_at).getTime()
    : 0;

  const activeGeofenceEvent = transitionEvents.find((event) => {
    const eventTimestamp = new Date(event.recorded_at).getTime();

    return Math.abs(eventTimestamp - currentTimestamp) <= 60_000;
  });

  const activeFuelEvent = fuelEvents.find((event) => {
    const eventTimestamp = new Date(event.event_time).getTime();

    return Math.abs(eventTimestamp - currentTimestamp) <= 60_000;
  });

  const activeAlert = alerts.find((alert) => {
    const alertTimestamp = new Date(alert.created_at).getTime();

    return Math.abs(alertTimestamp - currentTimestamp) <= 60_000;
  });

  const activeDeviceHealthEvent = deviceHealthEvents.find((event) => {
    const eventTimestamp = new Date(event.detected_at).getTime();

    return Math.abs(eventTimestamp - currentTimestamp) <= 60_000;
  });

  const activeDeviceStateEvent = deviceStateEvents.find((event) => {
    const eventTimestamp = new Date(event.recorded_at).getTime();

    return Math.abs(eventTimestamp - currentTimestamp) <= 60_000;
  });

  const replayEventFeed = [
    ...transitionEvents
      .filter(
        (event) => new Date(event.recorded_at).getTime() <= currentTimestamp,
      )
      .map((event) => ({
        timestamp: event.recorded_at,
        label:
          event.transition_type === "ENTERED_ZONE"
            ? `🟢 Entered ${event.geofence_name}`
            : `🔴 Exited ${event.geofence_name}`,
      })),

    ...fuelEvents
      .filter(
        (event) => new Date(event.event_time).getTime() <= currentTimestamp,
      )
      .map((event) => ({
        timestamp: event.event_time,
        label:
          event.event_type === "REFILL"
            ? `⛽ Refill ${event.fuel_difference.toFixed(2)}L`
            : event.event_type === "THEFT"
              ? `🚨 Theft ${event.fuel_difference.toFixed(2)}L`
              : `⚠ Leak ${event.fuel_difference.toFixed(2)}L`,
      })),

    ...alerts
      .filter(
        (alert) => new Date(alert.created_at).getTime() <= currentTimestamp,
      )
      .map((alert) => ({
        timestamp: alert.created_at,
        label:
          alert.severity === "Critical"
            ? `🚨 ${alert.alert_type}`
            : alert.severity === "Warning"
              ? `⚠ ${alert.alert_type}`
              : `ℹ ${alert.alert_type}`,
      })),

    ...deviceHealthEvents
      .filter(
        (event) => new Date(event.detected_at).getTime() <= currentTimestamp,
      )
      .map((event) => ({
        timestamp: event.detected_at,
        label:
          event.new_status === "ONLINE"
            ? "🟢 Online"
            : event.new_status === "STALE"
              ? "🟡 Stale"
              : event.new_status === "OFFLINE"
                ? "🔴 Offline"
                : "⚪ Unknown",
      })),

    ...deviceStateEvents
      .filter(
        (event) => new Date(event.recorded_at).getTime() <= currentTimestamp,
      )
      .map((event) => ({
        timestamp: event.recorded_at,
        label:
          event.state === "MOVING"
            ? "🚗 Moving"
            : event.state === "IDLE"
              ? "🟡 Idle"
              : "🅿️ Parked",
      })),
  ]
    .sort(
      (a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
    )
    .slice(0, 5);

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

      {activeGeofenceEvent && (
        <span>
          {activeGeofenceEvent.transition_type === "ENTERED_ZONE"
            ? "🟢 Entered"
            : "🔴 Exited"}{" "}
          {activeGeofenceEvent.geofence_name}
        </span>
      )}

      {activeFuelEvent && (
        <span>
          {activeFuelEvent.event_type === "REFILL"
            ? "⛽ Refill"
            : activeFuelEvent.event_type === "THEFT"
              ? "🚨 Theft"
              : "⚠ Leak"}{" "}
          {activeFuelEvent.fuel_difference.toFixed(2)}L
        </span>
      )}

      {activeAlert && (
        <span>
          {activeAlert.severity === "Critical"
            ? "🚨"
            : activeAlert.severity === "Warning"
              ? "⚠"
              : "ℹ"}{" "}
          {activeAlert.alert_type}
        </span>
      )}

      {activeDeviceHealthEvent && (
        <span>
          {activeDeviceHealthEvent.new_status === "ONLINE"
            ? "🟢 Online"
            : activeDeviceHealthEvent.new_status === "STALE"
              ? "🟡 Stale"
              : activeDeviceHealthEvent.new_status === "OFFLINE"
                ? "🔴 Offline"
                : "⚪ Unknown"}
        </span>
      )}

      {activeDeviceStateEvent && (
        <span>
          {activeDeviceStateEvent.state === "MOVING"
            ? "🚗 Moving"
            : activeDeviceStateEvent.state === "IDLE"
              ? "🟡 Idle"
              : "🅿️ Parked"}
        </span>
      )}

      <div style={{ marginTop: "0.75rem" }}>
        <strong>Recent Replay Events</strong>

        {replayEventFeed.length === 0 ? (
          <span>No replay events yet</span>
        ) : (
          replayEventFeed.map((event) => (
            <span key={`${event.timestamp}-${event.label}`}>
              {new Date(event.timestamp).toLocaleTimeString()} {event.label}
            </span>
          ))
        )}
      </div>

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
