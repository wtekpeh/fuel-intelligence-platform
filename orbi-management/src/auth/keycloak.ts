import Keycloak from "keycloak-js";

/**
 * Keycloak authenticates ORBI internal personnel.
 *
 * Authorization remains the responsibility of the
 * Rust backend and its platform-role checks.
 */
export const keycloak = new Keycloak({
  url: import.meta.env.VITE_KEYCLOAK_URL ?? "https://auth.williamtekpeh.com",

  realm: import.meta.env.VITE_KEYCLOAK_REALM ?? "orbi",

  clientId: import.meta.env.VITE_KEYCLOAK_CLIENT_ID ?? "orbi-management",
});
