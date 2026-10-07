import { keycloak } from "./keycloak";

/**
 * End the current customer's Keycloak session.
 *
 * Keycloak clears the authenticated session and redirects
 * the browser back to the ORBI application.
 *
 * Since ORBI uses login-required, returning to the application
 * will require authentication again.
 */
export async function logout(): Promise<void> {
  await keycloak.logout({
    redirectUri: window.location.origin,
  });
}
