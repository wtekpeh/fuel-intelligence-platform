import { httpClient } from "./httpClient";

export type OnboardingStatus =
  | "onboarding_required"
  | "active"
  | "platform_user";

export interface ClientOnboardingStatusResponse {
  status: OnboardingStatus;
  onboarding_complete: boolean;
  user_id: string | null;
}

export interface ClientOnboardingRequest {
  organization_name: string;
  industry: string;
}

export interface ClientOnboardingResponse {
  user_id: string;
  organization_id: string;
  message: string;
}

export async function getClientOnboardingStatus(): Promise<ClientOnboardingStatusResponse> {
  const response = await httpClient.get<ClientOnboardingStatusResponse>(
    "/api/onboarding/status",
  );

  return response.data;
}

export async function createClientOnboarding(
  payload: ClientOnboardingRequest,
): Promise<ClientOnboardingResponse> {
  const response = await httpClient.post<ClientOnboardingResponse>(
    "/api/onboarding/client",
    payload,
  );

  return response.data;
}
