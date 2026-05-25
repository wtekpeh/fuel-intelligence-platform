import { useEffect } from "react";

import { fetchOrganizationFleetOverview } from "../api/fleetApi";
import { useFleetStore } from "../store/fleetStore";

export function useFleetOverview(organizationId: string | null) {
  const setFleetItems = useFleetStore((state) => state.setFleetItems);

  useEffect(() => {
    if (!organizationId) {
      return;
    }

    const activeOrganizationId = organizationId;

    async function loadFleetOverview() {
      try {
        const fleetItems =
          await fetchOrganizationFleetOverview(activeOrganizationId);

        setFleetItems(fleetItems);
      } catch (error) {
        console.error(
          "[Fleet] Failed to load organization fleet overview.",
          error,
        );
      }
    }

    loadFleetOverview();
  }, [organizationId, setFleetItems]);
}
