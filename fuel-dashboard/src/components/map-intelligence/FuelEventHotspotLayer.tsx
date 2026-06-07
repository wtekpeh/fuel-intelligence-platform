import { CircleMarker, Popup } from "react-leaflet";
import { useInvestigationStore } from "../../store/investigationStore";

const EVENT_STYLES = {
  THEFT: {
    color: "#dc2626",
    fillColor: "#ef4444",
    label: "Theft Event",
  },
  REFILL: {
    color: "#16a34a",
    fillColor: "#22c55e",
    label: "Refill Event",
  },
  LEAK: {
    color: "#d97706",
    fillColor: "#f59e0b",
    label: "Leak Event",
  },
};

export default function FuelEventHotspotLayer() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const mappedEvents = fuelEvents.filter(
    (event) => event.latitude !== null && event.longitude !== null,
  );

  return (
    <>
      {mappedEvents.map((event) => {
        const style =
          EVENT_STYLES[event.event_type as keyof typeof EVENT_STYLES];

        if (!style) {
          return null;
        }

        return (
          <CircleMarker
            key={event.id}
            center={[event.latitude as number, event.longitude as number]}
            radius={20}
            pathOptions={{
              color: style.color,
              fillColor: style.fillColor,
              fillOpacity: 0.55,
              weight: 4,
            }}
          >
            <Popup>
              <div>
                <strong>{style.label}</strong>

                <div>Severity: {event.severity}</div>

                <div>Confidence: {event.confidence}</div>

                <div>
                  Fuel Change: {Math.abs(event.fuel_difference).toFixed(2)}L
                </div>
              </div>
            </Popup>
          </CircleMarker>
        );
      })}
    </>
  );
}
