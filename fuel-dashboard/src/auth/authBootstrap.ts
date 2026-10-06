import { keycloak } from "./keycloak";

/**
 * Initialize Keycloak before the ORBI application mounts.
 *
 * `login-required` means unauthenticated users are redirected to Keycloak.
 * Once authentication succeeds, Keycloak returns the browser to ORBI and
 * initialization resolves with an authenticated session.
 */
export async function initializeAuthentication(): Promise<boolean> {
  const authenticated = await keycloak.init({
    onLoad: "login-required",
    pkceMethod: "S256",
    checkLoginIframe: false,
  });

  return authenticated;
}
