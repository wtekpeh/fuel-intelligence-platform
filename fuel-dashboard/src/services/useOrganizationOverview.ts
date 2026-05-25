import { useEffect } from "react";

import { fetchOrganizationOverview } from "../api/organizationApi";
import { useOrganizationStore } from "../store/organizationStore";

export function useOrganizationOverview() {
  const setOrganizations = useOrganizationStore(
    (state) => state.setOrganizations,
  );

  useEffect(() => {
    async function loadOrganizations() {
      try {
        const organizations = await fetchOrganizationOverview();

        setOrganizations(organizations);
      } catch (error) {
        console.error("[Organizations] Failed to load overview.", error);
      }
    }

    loadOrganizations();
  }, [setOrganizations]);
}
