import { useFleetStore } from "../../store/fleetStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useTelemetryStore } from "../../store/telemetryStore";

const demoDepotBounds = {
  minLatitude: 8.5,
  maxLatitude: 10.5,
  minLongitude: 2.5,
  maxLongitude: 4.0,
};

function isInsideDemoDepot(latitude: number, longitude: number) {
  return (
    latitude >= demoDepotBounds.minLatitude &&
    latitude <= demoDepotBounds.maxLatitude &&
    longitude >= demoDepotBounds.minLongitude &&
    longitude <= demoDepotBounds.maxLongitude
  );
}

function GeofenceStatusCard() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);

  if (!selectedDevice) {
    return null;
  }

  const deviceReadings = readings
    .filter(
      (reading) =>
        reading.device_id === selectedDevice.device_id &&
        reading.latitude !== null &&
        reading.longitude !== null,
    )
    .sort(
      (a, b) =>
        new Date(a.recorded_at).getTime() - new Date(b.recorded_at).getTime(),
    );

  const activeReading = isReplayMode
    ? deviceReadings[currentIndex]
    : deviceReadings[deviceReadings.length - 1];

  if (
    !activeReading ||
    activeReading.latitude === null ||
    activeReading.longitude === null
  ) {
    return (
      <div className="map-live-card">
        <label>Geofence Status</label>
        <strong>No location available</strong>
      </div>
    );
  }

  const insideDepot = isInsideDemoDepot(
    activeReading.latitude,
    activeReading.longitude,
  );

  return (
    <div className="map-live-card">
      <label>Geofence Status</label>

      <strong>
        {insideDepot ? "Inside Depot Zone" : "Outside Depot Zone"}
      </strong>

      <span>
        {isReplayMode
          ? "Status follows the current replay position."
          : "Status follows the latest telemetry position."}
      </span>
    </div>
  );
}

export default GeofenceStatusCard;
