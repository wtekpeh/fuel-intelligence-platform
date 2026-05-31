import { useInvestigationStore } from "../../store/investigationStore";
import type {
  DeviceStateEvent,
  FuelEvent,
  SensorHealthEvent,
} from "../../types";

function FuelEventDetails({ event }: { event: FuelEvent }) {
  const telemetryTime = new Date(event.event_time);
  const detectionTime = new Date(event.detected_at);

  const telemetryAhead = telemetryTime.getTime() > detectionTime.getTime();

  const clockDifferenceMinutes = Math.abs(
    Math.round((telemetryTime.getTime() - detectionTime.getTime()) / 60000),
  );
  return (
    <div className="investigation-detail__grid">
      <div>
        <label>Fuel Before</label>
        <strong>{event.fuel_before.toFixed(2)}L</strong>
      </div>

      <div>
        <label>Fuel After</label>
        <strong>{event.fuel_after.toFixed(2)}L</strong>
      </div>

      <div>
        <label>Fuel Difference</label>
        <strong>{Math.abs(event.fuel_difference).toFixed(2)}L</strong>
      </div>

      <div>
        <label>Confidence</label>

        <strong
          className={`investigation-confidence investigation-confidence--${event.confidence.toLowerCase()}`}
        >
          {event.confidence}
        </strong>
      </div>

      <div>
        <label>Correlation Status</label>
        <strong>{event.correlation_status}</strong>
      </div>

      <div>
        <label>Operational Context</label>
        <strong>{event.correlation_reason}</strong>
      </div>

      <div>
        <label>Device Telemetry Time</label>
        <strong>{new Date(event.event_time).toLocaleString()}</strong>
      </div>

      <div>
        <label>Intelligence Detection Time</label>
        <strong>{new Date(event.detected_at).toLocaleString()}</strong>
      </div>

      <div>
        <label>Delayed Detection</label>
        <strong>{event.is_delayed_detection ? "Yes" : "No"}</strong>
      </div>

      <div>
        <label>Sync Delay</label>
        <strong>{event.sync_delay_seconds} seconds</strong>
      </div>

      {telemetryAhead && (
        <div className="investigation-clock-warning">
          <label>Telemetry Clock Check</label>

          <strong>
            Device telemetry timestamp appears ahead of backend intelligence
            detection by {clockDifferenceMinutes} minute(s).
          </strong>
        </div>
      )}
    </div>
  );
}

function DeviceStateDetails({ event }: { event: DeviceStateEvent }) {
  return (
    <div className="investigation-detail__grid">
      <div>
        <label>State</label>
        <strong>{event.state}</strong>
      </div>

      <div>
        <label>Motion</label>
        <strong>{event.motion_detected ? "Detected" : "Not detected"}</strong>
      </div>

      <div>
        <label>Vibration</label>
        <strong>{event.vibration_level ?? "N/A"}</strong>
      </div>

      <div>
        <label>GPS</label>
        <strong>
          {event.latitude && event.longitude
            ? `${event.latitude}, ${event.longitude}`
            : "N/A"}
        </strong>
      </div>
    </div>
  );
}

function SensorHealthDetails({ event }: { event: SensorHealthEvent }) {
  return (
    <div className="investigation-detail__grid">
      <div>
        <label>Sensor Event</label>
        <strong>{event.event_type}</strong>
      </div>

      <div>
        <label>Severity</label>
        <strong>{event.severity}</strong>
      </div>

      <div>
        <label>First Seen</label>
        <strong>{new Date(event.first_seen_at).toLocaleString()}</strong>
      </div>

      <div>
        <label>Last Seen</label>
        <strong>{new Date(event.last_seen_at).toLocaleString()}</strong>
      </div>

      <div>
        <label>Reason</label>
        <strong>{event.reason}</strong>
      </div>
    </div>
  );
}

export function InvestigationDetailPanel() {
  const selectedTimelineItem = useInvestigationStore(
    (state) => state.selectedTimelineItem,
  );

  const clearSelectedTimelineItem = useInvestigationStore(
    (state) => state.clearSelectedTimelineItem,
  );

  if (!selectedTimelineItem) {
    return null;
  }

  return (
    <aside className="investigation-detail">
      <div className="investigation-detail__header">
        <div>
          <span>{selectedTimelineItem.type.replace("_", " ")}</span>
          <h3>{selectedTimelineItem.title}</h3>
        </div>

        <button type="button" onClick={clearSelectedTimelineItem}>
          ×
        </button>
      </div>

      <p>{selectedTimelineItem.subtitle}</p>

      {selectedTimelineItem.type === "fuel_event" && (
        <FuelEventDetails event={selectedTimelineItem.raw as FuelEvent} />
      )}

      {selectedTimelineItem.type === "device_state" && (
        <DeviceStateDetails
          event={selectedTimelineItem.raw as DeviceStateEvent}
        />
      )}

      {selectedTimelineItem.type === "sensor_health" && (
        <SensorHealthDetails
          event={selectedTimelineItem.raw as SensorHealthEvent}
        />
      )}
    </aside>
  );
}
