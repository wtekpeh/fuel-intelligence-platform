import { useInvestigationStore } from "../../store/investigationStore";
import { useGeofenceStore } from "../../store/geofenceStore";

function ReplayEventSummaryCard() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const transitionEvents = useGeofenceStore((state) => state.transitionEvents);

  const theftEvents = fuelEvents.filter(
    (event) => event.event_type === "THEFT",
  );

  const refillEvents = fuelEvents.filter(
    (event) => event.event_type === "REFILL",
  );

  const leakEvents = fuelEvents.filter((event) => event.event_type === "LEAK");

  const enteredZones = transitionEvents.filter(
    (event) => event.transition_type === "ENTERED_ZONE",
  );

  const exitedZones = transitionEvents.filter(
    (event) => event.transition_type === "EXITED_ZONE",
  );

  return (
    <div className="map-live-card replay-event-summary-card">
      <label>Replay Event Summary</label>

      <div className="replay-event-summary-grid">
        <div>
          <label>Fuel Events</label>
          <strong>{fuelEvents.length}</strong>
        </div>

        <div>
          <label>Thefts</label>
          <strong>{theftEvents.length}</strong>
        </div>

        <div>
          <label>Refills</label>
          <strong>{refillEvents.length}</strong>
        </div>

        <div>
          <label>Leaks</label>
          <strong>{leakEvents.length}</strong>
        </div>

        <div>
          <label>Entries</label>
          <strong>{enteredZones.length}</strong>
        </div>

        <div>
          <label>Exits</label>
          <strong>{exitedZones.length}</strong>
        </div>
      </div>
    </div>
  );
}

export default ReplayEventSummaryCard;
