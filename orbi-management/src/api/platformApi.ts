import { httpClient } from "./httpClient";

export interface PlatformIdentity {
  user_id: string;
  username: string | null;
  email: string | null;
  platform_role: string;
}

export async function getPlatformIdentity(): Promise<PlatformIdentity> {
  const response = await httpClient.get<PlatformIdentity>("/api/platform/me");

  return response.data;
}
