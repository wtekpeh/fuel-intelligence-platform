import {
  getClientOnboardingStatus,
  type ClientOnboardingStatusResponse,
} from "../api/onboardingApi";

export type StartupDestination =
  | "dashboard"
  | "onboarding"
  | "platform"
  | "access_denied";

export async function resolveStartupDestination(): Promise<StartupDestination> {
  const response: ClientOnboardingStatusResponse =
    await getClientOnboardingStatus();

  switch (response.status) {
    case "onboarding_required":
      return "onboarding";

    case "active":
      return "dashboard";

    case "platform_user":
      return "platform";

    default:
      return "access_denied";
  }
}
