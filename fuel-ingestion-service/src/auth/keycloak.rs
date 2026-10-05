use anyhow::{Context, Result};
use jsonwebtoken::jwk::JwkSet;

/// Client responsible for retrieving Keycloak's public signing keys.
///
/// Keycloak signs access tokens with a private key. ORBI retrieves the
/// corresponding public keys from the realm's JWKS endpoint and later uses
/// them to verify access-token signatures locally.
#[derive(Debug, Clone)]
pub struct KeycloakClient {
    http_client: reqwest::Client,
    jwks_url: String,
}

impl KeycloakClient {
    /// Build a Keycloak JWKS client for one realm.
    pub fn new(keycloak_url: &str, realm: &str) -> Self {
        let base_url = keycloak_url.trim_end_matches('/');

        let jwks_url = format!("{base_url}/realms/{realm}/protocol/openid-connect/certs");

        Self {
            http_client: reqwest::Client::new(),
            jwks_url,
        }
    }

    /// Retrieve the current public signing keys published by Keycloak.
    pub async fn fetch_jwks(&self) -> Result<JwkSet> {
        let response = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await
            .context("failed to request Keycloak JWKS")?
            .error_for_status()
            .context("Keycloak JWKS request returned an unsuccessful status")?;

        response
            .json::<JwkSet>()
            .await
            .context("failed to decode Keycloak JWKS response")
    }
}
