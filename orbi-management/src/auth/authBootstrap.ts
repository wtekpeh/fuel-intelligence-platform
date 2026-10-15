import { keycloak } from "./keycloak";

/**
 * Authenticate ORBI internal management users.
 *
 * Keycloak verifies identity.
 * The Rust backend will separately verify SUPER_ADMIN access.
 */
export async function initializeAuthentication(): Promise<boolean> {
  const authenticated = await keycloak.init({
    onLoad: "login-required",
    pkceMethod: "S256",
    checkLoginIframe: false,
  });

  return authenticated;
}
