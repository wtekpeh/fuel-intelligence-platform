import { useEffect } from "react";

import { useGeofenceStore } from "../store/geofenceStore";
import { useOrganizationStore } from "../store/organizationStore";
import { useFleetStore } from "../store/fleetStore";

export function useGeofenceData() {
  const selectedOrganization = useOrganizationStore(
    (state) => state.selectedOrganization,
  );

  const loadGeofences = useGeofenceStore((state) => state.loadGeofences);

  const loadTransitionEvents = useGeofenceStore(
    (state) => state.loadTransitionEvents,
  );

  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  useEffect(() => {
    if (!selectedOrganization) {
      return;
    }

    loadGeofences(selectedOrganization.organization_id);
  }, [loadGeofences, selectedOrganization]);

  useEffect(() => {
    loadTransitionEvents(selectedDevice?.device_id);

    const intervalId = window.setInterval(() => {
      loadTransitionEvents(selectedDevice?.device_id);
    }, 5000);

    return () => {
      window.clearInterval(intervalId);
    };
  }, [loadTransitionEvents, selectedDevice]);
}
