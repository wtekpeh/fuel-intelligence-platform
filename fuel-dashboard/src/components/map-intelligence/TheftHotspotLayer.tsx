import { CircleMarker, Popup } from "react-leaflet";
import { useInvestigationStore } from "../../store/investigationStore";

export default function TheftHotspotLayer() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const theftEvents = fuelEvents.filter(
    (event) =>
      event.event_type === "THEFT" &&
      event.latitude !== null &&
      event.longitude !== null,
  );

  return (
    <>
      {theftEvents.map((event) => (
        <CircleMarker
          key={event.id}
          center={[event.latitude as number, event.longitude as number]}
          radius={20}
          pathOptions={{
            color: "#dc2626",
            fillColor: "#ef4444",
            fillOpacity: 0.55,
            weight: 4,
          }}
        >
          <Popup>
            <div>
              <strong>Theft Event</strong>

              <div>Severity: {event.severity}</div>

              <div>Confidence: {event.confidence}</div>

              <div>
                Fuel Loss: {Math.abs(event.fuel_difference).toFixed(2)}L
              </div>
            </div>
          </Popup>
        </CircleMarker>
      ))}
    </>
  );
}
