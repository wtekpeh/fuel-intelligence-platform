import { useEffect } from "react";

import { useGeofenceStore } from "../store/geofenceStore";
import { useOrganizationStore } from "../store/organizationStore";

export function useGeofenceData() {
  const selectedOrganization = useOrganizationStore(
    (state) => state.selectedOrganization,
  );

  const loadGeofences = useGeofenceStore((state) => state.loadGeofences);

  useEffect(() => {
    if (!selectedOrganization) {
      return;
    }

    loadGeofences(selectedOrganization.organization_id);
  }, [loadGeofences, selectedOrganization]);
}
