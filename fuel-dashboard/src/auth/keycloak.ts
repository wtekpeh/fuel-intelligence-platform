import Keycloak from "keycloak-js";

/*
 * Keycloak is responsible for human identity authentication.
 *
 * ORBI-specific authorization, roles, permissions, and organization
 * membership remain application concerns and are handled separately.
 */
export const keycloak = new Keycloak({
  url: import.meta.env.VITE_KEYCLOAK_URL ?? "https://auth.williamtekpeh.com",

  realm: import.meta.env.VITE_KEYCLOAK_REALM ?? "orbi",

  clientId: import.meta.env.VITE_KEYCLOAK_CLIENT_ID ?? "orbi-dashboard",
});
