import { CircleMarker, Polyline, Popup } from "react-leaflet";

import { useFleetStore } from "../../store/fleetStore";
import { useTelemetryStore } from "../../store/telemetryStore";
import { useMapReplayStore } from "../../store/mapReplayStore";

function TelemetryRouteLayer() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const readings = useTelemetryStore((state) => state.readings);

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);

  const currentIndex = useMapReplayStore((state) => state.currentIndex);

  const replayReadings = useMapReplayStore((state) => state.replayReadings);

  if (!selectedDevice) {
    return null;
  }

  const sourceReadings = replayReadings.length > 0 ? replayReadings : readings;

  const deviceReadings = sourceReadings
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

  const routeCoordinates = deviceReadings.map((reading) => [
    reading.latitude,
    reading.longitude,
  ]) as [number, number][];

  if (routeCoordinates.length < 2) {
    return null;
  }

  const firstReading = deviceReadings[0];
  const latestReading = deviceReadings[deviceReadings.length - 1];

  const breadcrumbReadings = deviceReadings.slice(1, -1);

  const replayedCoordinates = routeCoordinates.slice(0, currentIndex + 1);

  return (
    <>
      <Polyline
        positions={routeCoordinates}
        pathOptions={{
          color: "#334155",
          weight: 4,
          opacity: 0.35,
        }}
      />

      {isReplayMode && replayedCoordinates.length >= 2 && (
        <Polyline
          positions={replayedCoordinates}
          pathOptions={{
            color: "#38bdf8",
            weight: 5,
            opacity: 0.92,
          }}
        />
      )}

      {breadcrumbReadings.map((reading, index) => {
        const isReplayed = index <= currentIndex;

        return (
          <CircleMarker
            key={`${reading.device_id}-${reading.recorded_at}`}
            center={[reading.latitude as number, reading.longitude as number]}
            radius={isReplayed ? 5 : 4}
            pathOptions={{
              color: isReplayed ? "#f8fafc" : "#bae6fd",
              fillColor: isReplayed ? "#38bdf8" : "#64748b",
              fillOpacity: isReplayed ? 0.92 : 0.45,
              opacity: 0.9,
              weight: isReplayed ? 2 : 1,
            }}
          >
            <Popup>
              <strong>Telemetry Breadcrumb</strong>
              <br />
              {new Date(reading.recorded_at).toLocaleString()}
              <br />
              Fuel: {reading.fuel_level_litres.toFixed(2)}L
            </Popup>
          </CircleMarker>
        );
      })}

      <CircleMarker
        center={[
          firstReading.latitude as number,
          firstReading.longitude as number,
        ]}
        radius={8}
        pathOptions={{
          color: "#22c55e",
          fillColor: "#22c55e",
          fillOpacity: 0.85,
          weight: 3,
        }}
      >
        <Popup>
          <strong>Route Start</strong>
          <br />
          {new Date(firstReading.recorded_at).toLocaleString()}
        </Popup>
      </CircleMarker>

      <CircleMarker
        center={[
          latestReading.latitude as number,
          latestReading.longitude as number,
        ]}
        radius={10}
        pathOptions={{
          color: "#f8fafc",
          fillColor: "#38bdf8",
          fillOpacity: 0.9,
          weight: 3,
        }}
      >
        <Popup>
          <strong>Latest Route Point</strong>
          <br />
          {new Date(latestReading.recorded_at).toLocaleString()}
          <br />
          Fuel: {latestReading.fuel_level_litres.toFixed(2)}L
        </Popup>
      </CircleMarker>
    </>
  );
}

export default TelemetryRouteLayer;
