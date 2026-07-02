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

export interface CreateOrganizationRequest {
  name: string;
  industry?: string | null;
}

export interface OrganizationMutationResponse {
  organization_id: string;
  message: string;
}

export async function createOrganization(
  request: CreateOrganizationRequest,
): Promise<OrganizationMutationResponse> {
  const response = await httpClient.post<OrganizationMutationResponse>(
    "/api/organizations",
    request,
  );

  return response.data;
}
