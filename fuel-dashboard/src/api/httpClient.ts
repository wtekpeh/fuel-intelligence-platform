import axios from "axios";

import { keycloak } from "../auth/keycloak";

export const httpClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:8080",
  headers: {
    "Content-Type": "application/json",
  },
});

/*
 * Human-facing ORBI API requests authenticate with the current Keycloak
 * access token.
 *
 * Authentication initialization completes before <App /> mounts, so normal
 * operational requests begin only after the Keycloak session is established.
 */
httpClient.interceptors.request.use(async (config) => {
  /*
   * Refresh the access token when it has less than 30 seconds remaining.
   *
   * updateToken() does not necessarily contact Keycloak on every request.
   * If the current token is still sufficiently valid, it resolves without
   * refreshing it.
   */
  await keycloak.updateToken(30);

  if (keycloak.token) {
    config.headers.Authorization = `Bearer ${keycloak.token}`;
  }

  return config;
});
