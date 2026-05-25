import { httpClient } from "./httpClient";

import type { OrganizationOverview } from "../types";

export async function fetchOrganizationOverview(): Promise<
  OrganizationOverview[]
> {
  const response = await httpClient.get<OrganizationOverview[]>(
    "/api/organizations/overview",
  );

  return response.data;
}
