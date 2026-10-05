use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;

use super::verifier::KeycloakTokenVerifier;

/// Shared runtime authentication state for ORBI's human-user API.
///
/// Keycloak public signing keys are cached locally so normal authenticated
/// requests can verify access tokens without making a network request to
/// Keycloak.
///
/// The cache uses an RwLock because token verification only needs shared
/// read access. Exclusive write access will be required only when the JWKS
/// cache is refreshed, for example after Keycloak rotates signing keys.
#[derive(Debug)]
pub struct AuthState {
    pub verifier: KeycloakTokenVerifier,
    pub jwks: RwLock<JwkSet>,
}

impl AuthState {
    pub fn new(verifier: KeycloakTokenVerifier, jwks: JwkSet) -> Self {
        Self {
            verifier,
            jwks: RwLock::new(jwks),
        }
    }
}
