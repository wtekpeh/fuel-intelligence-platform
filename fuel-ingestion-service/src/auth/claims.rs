use serde::Deserialize;

/// Claims ORBI expects from a successfully verified Keycloak access token.
///
/// This structure represents identity information supplied by Keycloak.
/// ORBI-specific organization membership and authorization are intentionally
/// kept separate from these external identity claims.
#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakClaims {
    /// Stable Keycloak identifier for the authenticated user.
    pub sub: String,

    /// Token issuer. This must eventually match the configured ORBI
    /// Keycloak realm issuer.
    pub iss: String,

    /// Expiration time represented as Unix time.
    pub exp: usize,

    /// Time at which the token was issued, when supplied by Keycloak.
    #[serde(default)]
    pub iat: Option<usize>,

    /// Human-readable username, when supplied by Keycloak.
    #[serde(default)]
    pub preferred_username: Option<String>,

    /// User email, when supplied by Keycloak.
    #[serde(default)]
    pub email: Option<String>,
}
