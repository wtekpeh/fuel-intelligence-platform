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
        <label>Geofence Status</label>
        <strong>No location available</strong>
      </div>
    );
  }

  if (!positionStatus) {
    return (
      <div className="map-live-card">
        <label>Geofence Status</label>
        <strong>Checking zone...</strong>
      </div>
    );
  }

  return (
    <div className="map-live-card">
      <label>Geofence Status</label>

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
    </div>
  );
}

export default GeofenceStatusCard;
