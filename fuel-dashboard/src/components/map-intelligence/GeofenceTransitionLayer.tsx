import { CircleMarker, Popup } from "react-leaflet";

import { useGeofenceStore } from "../../store/geofenceStore";
import { useInvestigationStore } from "../../store/investigationStore";

const ENTER_COLOR = "#22c55e";
const EXIT_COLOR = "#ef4444";

export default function GeofenceTransitionLayer() {
  const { transitionEvents } = useGeofenceStore();

  const { selectedTimelineItem, selectTimelineItem } = useInvestigationStore();

  return (
    <>
      {transitionEvents.map((event) => {
        const isSelected = selectedTimelineItem?.id === `geofence-${event.id}`;

        const color =
          event.transition_type === "ENTERED_ZONE" ? ENTER_COLOR : EXIT_COLOR;

        return (
          <CircleMarker
            key={event.id}
            center={[event.latitude, event.longitude]}
            radius={isSelected ? 16 : 11}
            pathOptions={{
              color,
              fillColor: color,
              fillOpacity: 0.95,
              weight: isSelected ? 5 : 3,
            }}
            eventHandlers={{
              click: () => {
                selectTimelineItem({
                  id: `geofence-${event.id}`,
                  type: "geofence_transition",
                  timestamp: event.recorded_at,
                  title: event.transition_type,
                  subtitle: `${event.geofence_name} (${event.geofence_type})`,
                  severity:
                    event.transition_type === "ENTERED_ZONE"
                      ? "good"
                      : "warning",
                  raw: event,
                });
              },
            }}
          >
            <Popup>
              <div>
                <strong>{event.transition_type}</strong>
                <br />
                <strong>Geofence:</strong> {event.geofence_name}
                <br />
                <strong>Type:</strong> {event.geofence_type}
                <br />
                <strong>Recorded:</strong>
                <br />
                {new Date(event.recorded_at).toLocaleString()}
              </div>
            </Popup>
          </CircleMarker>
        );
      })}
    </>
  );
}
