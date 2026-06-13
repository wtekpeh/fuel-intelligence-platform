import { useMapReplayStore } from "../../store/mapReplayStore";
import { lineString } from "@turf/helpers";
import length from "@turf/length";
import { useGeofenceStore } from "../../store/geofenceStore";

function formatDuration(startTime: string, endTime: string) {
  const durationMs =
    new Date(endTime).getTime() - new Date(startTime).getTime();

  const totalMinutes = Math.max(Math.floor(durationMs / 60_000), 0);

  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) {
    return `${minutes}m`;
  }

  return `${hours}h ${minutes}m`;
}

export default function JourneyIntelligenceCard() {
  const replayReadings = useMapReplayStore((state) => state.replayReadings);

  const transitionEvents = useGeofenceStore((state) => state.transitionEvents);

  const journeyReadings = replayReadings.filter(
    (reading) => reading.latitude !== null && reading.longitude !== null,
  );

  const zoneVisitCounts = transitionEvents.reduce<Record<string, number>>(
    (counts, event) => {
      const currentCount = counts[event.geofence_name] ?? 0;

      counts[event.geofence_name] = currentCount + 1;

      return counts;
    },
    {},
  );

  const visitedZones = Object.entries(zoneVisitCounts).sort(
    (firstZone, secondZone) => secondZone[1] - firstZone[1],
  );

  const totalZoneTransitions = transitionEvents.length;

  const mostActiveZone = visitedZones.length > 0 ? visitedZones[0] : null;

  if (journeyReadings.length < 2) {
    return (
      <div className="map-live-card">
        <label>Journey Intelligence</label>
        <strong>No journey loaded</strong>
        <span>
          Load Today, Yesterday, Last 7 Days, or a custom range to calculate
          journey distance and duration.
        </span>
      </div>
    );
  }

  const routeLine = lineString(
    journeyReadings.map((reading) => [
      reading.longitude as number,
      reading.latitude as number,
    ]),
  );

  const totalDistanceKm = length(routeLine, {
    units: "kilometers",
  });

  const firstReading = journeyReadings[0];
  const lastReading = journeyReadings[journeyReadings.length - 1];

  return (
    <div className="map-live-card journey-intelligence-card">
      <label>Journey Intelligence</label>

      <strong>{totalDistanceKm.toFixed(2)} km</strong>

      <span>
        Duration:{" "}
        {formatDuration(firstReading.recorded_at, lastReading.recorded_at)}
      </span>

      <span>Replay Points: {journeyReadings.length}</span>

      <span>Total Zone Transitions: {totalZoneTransitions}</span>

      {mostActiveZone && (
        <span>
          Most Active Zone: {mostActiveZone[0]} ({mostActiveZone[1]})
        </span>
      )}

      {visitedZones.length > 0 && (
        <>
          <span>Visited Zones</span>

          {visitedZones.map(([zoneName, visitCount]) => (
            <span key={zoneName}>
              {zoneName} ({visitCount})
            </span>
          ))}
        </>
      )}

      <span>
        Last Destination: {(lastReading.latitude as number).toFixed(5)},{" "}
        {(lastReading.longitude as number).toFixed(5)}
      </span>
    </div>
  );
}
