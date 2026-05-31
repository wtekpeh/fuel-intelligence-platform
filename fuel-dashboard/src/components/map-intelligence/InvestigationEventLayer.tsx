import { CircleMarker, Popup } from "react-leaflet";

import { useInvestigationStore } from "../../store/investigationStore";
import type { FuelEvent } from "../../types";

function getFuelEventPathOptions(severity: string) {
  const normalizedSeverity = severity.toLowerCase();

  if (normalizedSeverity === "critical" || normalizedSeverity === "high") {
    return {
      color: "#ef4444",
      fillColor: "#ef4444",
      fillOpacity: 0.55,
      radius: 9,
    };
  }

  if (normalizedSeverity === "medium") {
    return {
      color: "#f59e0b",
      fillColor: "#f59e0b",
      fillOpacity: 0.5,
      radius: 8,
    };
  }

  return {
    color: "#38bdf8",
    fillColor: "#38bdf8",
    fillOpacity: 0.45,
    radius: 7,
  };
}

function mapFuelEventSeverityToTimelineSeverity(
  severity: string,
): "good" | "warning" | "danger" | "neutral" {
  const normalizedSeverity = severity.toLowerCase();

  if (normalizedSeverity === "critical" || normalizedSeverity === "high") {
    return "danger";
  }

  if (normalizedSeverity === "medium") {
    return "warning";
  }

  if (normalizedSeverity === "low") {
    return "good";
  }

  return "neutral";
}

function InvestigationEventLayer() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const selectTimelineItem = useInvestigationStore(
    (state) => state.selectTimelineItem,
  );

  const setFocusedFuelEventId = useInvestigationStore(
    (state) => state.setFocusedFuelEventId,
  );

  const fuelEventsWithLocation = fuelEvents.filter(
    (event) => event.latitude !== null && event.longitude !== null,
  );

  function handleFuelEventClick(event: FuelEvent) {
    setFocusedFuelEventId(event.id);

    selectTimelineItem({
      id: `fuel-${event.id}`,
      type: "fuel_event",
      title: event.event_type,
      subtitle: event.message,
      timestamp: event.event_time,
      severity: mapFuelEventSeverityToTimelineSeverity(event.severity),
      raw: event,
    });
  }

  return (
    <>
      {fuelEventsWithLocation.map((event) => (
        <CircleMarker
          key={event.id}
          center={[event.latitude as number, event.longitude as number]}
          pathOptions={getFuelEventPathOptions(event.severity)}
          eventHandlers={{
            click: () => handleFuelEventClick(event),
          }}
        >
          <Popup>
            <div>
              <strong>{event.event_type}</strong>

              <br />

              <span>{event.severity}</span>

              <hr />

              <div>Fuel Before: {event.fuel_before.toFixed(2)}L</div>

              <div>Fuel After: {event.fuel_after.toFixed(2)}L</div>

              <div>
                Difference: {Math.abs(event.fuel_difference).toFixed(2)}L
              </div>

              <div>
                Event Time: {new Date(event.event_time).toLocaleString()}
              </div>

              <div>
                Detection Time: {new Date(event.detected_at).toLocaleString()}
              </div>

              <div>Context: {event.correlation_reason}</div>
            </div>
          </Popup>
        </CircleMarker>
      ))}
    </>
  );
}

export default InvestigationEventLayer;
