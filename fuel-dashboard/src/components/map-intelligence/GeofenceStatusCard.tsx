import { useEffect } from "react";

import { useFleetStore } from "../../store/fleetStore";
import { useGeofenceStore } from "../../store/geofenceStore";
import { useMapReplayStore } from "../../store/mapReplayStore";
import { useOrganizationStore } from "../../store/organizationStore";
import { useTelemetryStore } from "../../store/telemetryStore";

function GeofenceStatusCard() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);
  const readings = useTelemetryStore((state) => state.readings);

  const selectedOrganization = useOrganizationStore(
    (state) => state.selectedOrganization,
  );

  const isReplayMode = useMapReplayStore((state) => state.isReplayMode);
  const currentIndex = useMapReplayStore((state) => state.currentIndex);

  const positionStatus = useGeofenceStore((state) => state.positionStatus);

  const geofences = useGeofenceStore((state) => state.geofences);

  const transitionEvents = useGeofenceStore((state) => state.transitionEvents);

  const checkCurrentPosition = useGeofenceStore(
    (state) => state.checkCurrentPosition,
  );

  const deviceReadings = selectedDevice
    ? readings
        .filter(
          (reading) =>
            reading.device_id === selectedDevice.device_id &&
            reading.latitude !== null &&
            reading.longitude !== null,
        )
        .sort(
          (a, b) =>
            new Date(a.recorded_at).getTime() -
            new Date(b.recorded_at).getTime(),
        )
    : [];

  const activeReading = isReplayMode
    ? deviceReadings[currentIndex]
    : deviceReadings[deviceReadings.length - 1];

  useEffect(() => {
    if (
      !selectedOrganization ||
      !selectedDevice ||
      !activeReading ||
      activeReading.latitude === null ||
      activeReading.longitude === null
    ) {
      return;
    }

    checkCurrentPosition(
      selectedOrganization.organization_id,
      selectedDevice.device_id,
      activeReading.latitude,
      activeReading.longitude,
    );
  }, [
    activeReading,
    checkCurrentPosition,
    selectedDevice,
    selectedOrganization,
  ]);

  const totalGeofences = geofences.length;

  const totalEntries = transitionEvents.filter(
    (event) => event.transition_type === "ENTERED_ZONE",
  ).length;

  const totalExits = transitionEvents.filter(
    (event) => event.transition_type === "EXITED_ZONE",
  ).length;

  const latestTransition =
    transitionEvents.length > 0 ? transitionEvents[0] : null;

  const zoneActivityCounts = transitionEvents.reduce<
    Record<string, { name: string; count: number }>
  >((counts, event) => {
    const existingZone = counts[event.geofence_id];

    if (!existingZone) {
      counts[event.geofence_id] = {
        name: event.geofence_name,
        count: 1,
      };

      return counts;
    }

    existingZone.count += 1;

    return counts;
  }, {});

  const mostActiveZone = Object.values(zoneActivityCounts).sort(
    (firstZone, secondZone) => secondZone.count - firstZone.count,
  )[0];

  const totalTransitions = transitionEvents.length;

  const mostActiveZonePercentage =
    mostActiveZone && totalTransitions > 0
      ? ((mostActiveZone.count / totalTransitions) * 100).toFixed(0)
      : null;

  const mostActiveZoneConcentration =
    mostActiveZonePercentage === null
      ? null
      : Number(mostActiveZonePercentage) >= 76
        ? "Operational Dependency"
        : Number(mostActiveZonePercentage) >= 51
          ? "High Concentration"
          : Number(mostActiveZonePercentage) >= 26
            ? "Moderate Concentration"
            : "Distributed Activity";

  if (!selectedDevice) {
    return null;
  }

  if (
    !activeReading ||
    activeReading.latitude === null ||
    activeReading.longitude === null
  ) {
    return (
      <div className="map-live-card">
        <label>Geofence Intelligence</label>
        <strong>No location available</strong>
      </div>
    );
  }

  if (!positionStatus) {
    return (
      <div className="map-live-card">
        <label>Geofence Intelligence</label>
        <strong>Checking zone...</strong>
      </div>
    );
  }

  return (
    <div className="map-live-card">
      <label>Geofence Intelligence</label>

      <strong>
        {positionStatus.inside_geofence
          ? "Inside Operational Zone"
          : "Outside Operational Zones"}
      </strong>

      {positionStatus.matched_geofences.length > 0 && (
        <span>
          {positionStatus.matched_geofences
            .map(
              (geofence) =>
                `${geofence.geofence_name} (${geofence.geofence_type})`,
            )
            .join(", ")}
        </span>
      )}

      <span>
        {isReplayMode
          ? "Status follows the current replay position using PostGIS."
          : "Status follows the latest telemetry position using PostGIS."}
      </span>

      <div className="geofence-intelligence-grid">
        <div>
          <label>Total Zones</label>
          <strong>{totalGeofences}</strong>
        </div>

        <div>
          <label>Entries</label>
          <strong>{totalEntries}</strong>
        </div>

        <div>
          <label>Exits</label>
          <strong>{totalExits}</strong>
        </div>
      </div>

      {latestTransition && (
        <span>Latest Transition: {latestTransition.transition_type}</span>
      )}

      {mostActiveZone && (
        <div className="geofence-intelligence-highlight">
          <label>Most Active Zone</label>

          <strong>{mostActiveZone.name}</strong>

          <span>
            {mostActiveZone.count} transitions · {mostActiveZonePercentage}% of
            activity
          </span>

          {mostActiveZoneConcentration && (
            <small>{mostActiveZoneConcentration}</small>
          )}
        </div>
      )}
    </div>
  );
}

export default GeofenceStatusCard;
