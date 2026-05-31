import { useEffect, useRef } from "react";
import { CircleMarker, Popup } from "react-leaflet";
import type { CircleMarker as LeafletCircleMarker } from "leaflet";

import { useInvestigationStore } from "../../store/investigationStore";
import type { FuelEvent } from "../../types";

function getFuelEventPathOptions(severity: string, isFocused: boolean) {
  const normalizedSeverity = severity.toLowerCase();

  if (isFocused) {
    return {
      color: "#38bdf8",
      fillColor: "#38bdf8",
      fillOpacity: 0.85,
      opacity: 1,
      weight: 4,
    };
  }

  if (normalizedSeverity === "critical" || normalizedSeverity === "high") {
    return {
      color: "#ef4444",
      fillColor: "#ef4444",
      fillOpacity: 0.7,
      opacity: 1,
      weight: 3,
    };
  }

  if (normalizedSeverity === "medium") {
    return {
      color: "#f59e0b",
      fillColor: "#f59e0b",
      fillOpacity: 0.65,
      opacity: 1,
      weight: 3,
    };
  }

  return {
    color: "#38bdf8",
    fillColor: "#38bdf8",
    fillOpacity: 0.55,
    opacity: 1,
    weight: 2,
  };
}

function getFuelEventRadius(severity: string, isFocused: boolean) {
  if (isFocused) {
    return 18;
  }

  const normalizedSeverity = severity.toLowerCase();

  if (normalizedSeverity === "critical" || normalizedSeverity === "high") {
    return 13;
  }

  if (normalizedSeverity === "medium") {
    return 11;
  }

  return 9;
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

function FuelEventMarker({
  event,
  isFocused,
  onSelect,
}: {
  event: FuelEvent;
  isFocused: boolean;
  onSelect: (event: FuelEvent) => void;
}) {
  const markerRef = useRef<LeafletCircleMarker | null>(null);

  useEffect(() => {
    if (!isFocused) {
      return;
    }

    markerRef.current?.openPopup();
  }, [isFocused]);

  if (event.latitude === null || event.longitude === null) {
    return null;
  }

  return (
    <CircleMarker
      ref={markerRef}
      center={[event.latitude, event.longitude]}
      radius={getFuelEventRadius(event.severity, isFocused)}
      pathOptions={getFuelEventPathOptions(event.severity, isFocused)}
      eventHandlers={{
        click: () => onSelect(event),
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

          <div>Difference: {Math.abs(event.fuel_difference).toFixed(2)}L</div>

          <div>Event Time: {new Date(event.event_time).toLocaleString()}</div>

          <div>
            Detection Time: {new Date(event.detected_at).toLocaleString()}
          </div>

          <div>Context: {event.correlation_reason}</div>
        </div>
      </Popup>
    </CircleMarker>
  );
}

function InvestigationEventLayer() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const selectedTimelineItem = useInvestigationStore(
    (state) => state.selectedTimelineItem,
  );

  const selectTimelineItem = useInvestigationStore(
    (state) => state.selectTimelineItem,
  );

  const setFocusedFuelEventId = useInvestigationStore(
    (state) => state.setFocusedFuelEventId,
  );

  const focusedFuelEventId =
    selectedTimelineItem?.type === "fuel_event"
      ? (selectedTimelineItem.raw as FuelEvent).id
      : null;

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

  const fuelEventsWithLocation = fuelEvents.filter(
    (event) => event.latitude !== null && event.longitude !== null,
  );

  return (
    <>
      {fuelEventsWithLocation.map((event) => (
        <FuelEventMarker
          key={event.id}
          event={event}
          isFocused={event.id === focusedFuelEventId}
          onSelect={handleFuelEventClick}
        />
      ))}
    </>
  );
}

export default InvestigationEventLayer;
