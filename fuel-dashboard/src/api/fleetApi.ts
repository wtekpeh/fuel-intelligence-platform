import { httpClient } from "./httpClient";

import type { OrganizationFleetOverview } from "../types";

export async function fetchOrganizationFleetOverview(
  organizationId: string,
): Promise<OrganizationFleetOverview[]> {
  const response = await httpClient.get<OrganizationFleetOverview[]>(
    `/api/organizations/${organizationId}/fleet-overview`,
  );

  return response.data;
}
