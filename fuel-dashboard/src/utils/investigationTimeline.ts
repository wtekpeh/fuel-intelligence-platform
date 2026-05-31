import type { DeviceStateEvent, FuelEvent, SensorHealthEvent } from "../types";

export type InvestigationTimelineItemType =
  | "fuel_event"
  | "device_state"
  | "sensor_health";

export interface InvestigationTimelineItem {
  id: string;
  type: InvestigationTimelineItemType;
  timestamp: string;
  title: string;
  subtitle: string;
  severity: "good" | "warning" | "danger" | "neutral";
  raw: FuelEvent | DeviceStateEvent | SensorHealthEvent;
}

export function buildInvestigationTimeline(params: {
  fuelEvents: FuelEvent[];
  deviceStateEvents: DeviceStateEvent[];
  sensorHealthEvents: SensorHealthEvent[];
}): InvestigationTimelineItem[] {
  const fuelItems: InvestigationTimelineItem[] = params.fuelEvents.map(
    (event) => ({
      id: `fuel-${event.id}`,
      type: "fuel_event",
      timestamp: event.event_time,
      title: event.event_type,
      subtitle: buildFuelEventSubtitle(event),
      severity:
        event.confidence === "Critical" || event.severity === "critical"
          ? "danger"
          : event.confidence === "High"
            ? "warning"
            : "neutral",
      raw: event,
    }),
  );

  const stateItems: InvestigationTimelineItem[] = params.deviceStateEvents.map(
    (event, index) => ({
      id: `state-${event.recorded_at}-${index}`,
      type: "device_state",
      timestamp: event.recorded_at,
      title: event.state,
      subtitle: event.motion_detected
        ? "Motion detected"
        : "No motion detected",
      severity:
        event.state === "MOVING"
          ? "good"
          : event.state === "PARKED"
            ? "neutral"
            : "warning",
      raw: event,
    }),
  );

  const sensorItems: InvestigationTimelineItem[] =
    params.sensorHealthEvents.map((event) => ({
      id: `sensor-${event.id}`,
      type: "sensor_health",
      timestamp: event.detected_at,
      title: event.event_type,
      subtitle: event.reason,
      severity:
        event.severity === "critical"
          ? "danger"
          : event.severity === "medium"
            ? "warning"
            : "neutral",
      raw: event,
    }));

  return [...fuelItems, ...stateItems, ...sensorItems].sort(
    (first, second) =>
      new Date(second.timestamp).getTime() -
      new Date(first.timestamp).getTime(),
  );
}

function buildFuelEventSubtitle(event: FuelEvent) {
  const fuelChange = Math.abs(event.fuel_difference).toFixed(2);

  if (event.event_type === "REFILL") {
    return `${fuelChange}L fuel increase detected. Correlation: ${event.correlation_status}.`;
  }

  if (event.event_type === "THEFT") {
    return `${fuelChange}L fuel drop detected. Correlation: ${event.correlation_status}.`;
  }

  if (event.event_type === "LEAK") {
    return `${fuelChange}L gradual fuel loss detected. Correlation: ${event.correlation_status}.`;
  }

  return event.message;
}
