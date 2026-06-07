import { CircleMarker, Popup } from "react-leaflet";
import { useInvestigationStore } from "../../store/investigationStore";

interface HotspotCluster {
  id: string;
  latitude: number;
  longitude: number;
  eventCount: number;
  totalFuelChange: number;
  eventTypes: string[];
}

const CLUSTER_DISTANCE_DEGREES = 0.01;

function isNearCluster(
  eventLatitude: number,
  eventLongitude: number,
  cluster: HotspotCluster,
) {
  const latitudeDifference = Math.abs(eventLatitude - cluster.latitude);

  const longitudeDifference = Math.abs(eventLongitude - cluster.longitude);

  return (
    latitudeDifference <= CLUSTER_DISTANCE_DEGREES &&
    longitudeDifference <= CLUSTER_DISTANCE_DEGREES
  );
}

export default function HotspotClusterLayer() {
  const fuelEvents = useInvestigationStore((state) => state.fuelEvents);

  const mappedFuelEvents = fuelEvents.filter(
    (event) =>
      event.latitude !== null &&
      event.longitude !== null &&
      ["THEFT", "REFILL", "LEAK"].includes(event.event_type),
  );

  const clusters = mappedFuelEvents.reduce<HotspotCluster[]>(
    (existingClusters, event) => {
      const eventLatitude = event.latitude as number;
      const eventLongitude = event.longitude as number;

      const matchingCluster = existingClusters.find((cluster) =>
        isNearCluster(eventLatitude, eventLongitude, cluster),
      );

      if (!matchingCluster) {
        existingClusters.push({
          id: event.id,
          latitude: eventLatitude,
          longitude: eventLongitude,
          eventCount: 1,
          totalFuelChange: Math.abs(event.fuel_difference),
          eventTypes: [event.event_type],
        });

        return existingClusters;
      }

      matchingCluster.eventCount += 1;
      matchingCluster.totalFuelChange += Math.abs(event.fuel_difference);

      if (!matchingCluster.eventTypes.includes(event.event_type)) {
        matchingCluster.eventTypes.push(event.event_type);
      }

      matchingCluster.latitude = (matchingCluster.latitude + eventLatitude) / 2;

      matchingCluster.longitude =
        (matchingCluster.longitude + eventLongitude) / 2;

      return existingClusters;
    },
    [],
  );

  return (
    <>
      {clusters.map((cluster) => (
        <CircleMarker
          key={cluster.id}
          center={[cluster.latitude, cluster.longitude]}
          radius={Math.min(12 + cluster.eventCount * 5, 34)}
          pathOptions={{
            color: "#f97316",
            fillColor: "#fb923c",
            fillOpacity: 0.45,
            weight: 4,
          }}
        >
          <Popup>
            <div>
              <strong>Operational Hotspot</strong>

              <div>Events: {cluster.eventCount}</div>

              <div>Event Types: {cluster.eventTypes.join(", ")}</div>

              <div>
                Total Fuel Change: {cluster.totalFuelChange.toFixed(2)}L
              </div>
            </div>
          </Popup>
        </CircleMarker>
      ))}
    </>
  );
}
