use anyhow::{Context, Result, bail};
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm},
};

use super::claims::KeycloakClaims;

/// Verifies Keycloak access tokens against a previously retrieved JWKS.
///
/// JWKS retrieval is intentionally separate from verification so that
/// public signing keys can later be cached instead of fetched for every
/// authenticated API request.
#[derive(Debug, Clone)]
pub struct KeycloakTokenVerifier {
    issuer: String,
    audience: String,
}

impl KeycloakTokenVerifier {
    pub fn new(keycloak_url: &str, realm: &str, audience: &str) -> Self {
        let base_url = keycloak_url.trim_end_matches('/');

        Self {
            issuer: format!("{base_url}/realms/{realm}"),
            audience: audience.to_string(),
        }
    }

    /// Verify a Keycloak access token using the supplied realm JWKS.
    pub fn verify(&self, token: &str, jwks: &JwkSet) -> Result<KeycloakClaims> {
        let header = decode_header(token).context("failed to decode JWT header")?;

        let kid = header
            .kid
            .as_deref()
            .context("JWT header does not contain a key ID")?;

        /*
         * ORBI currently accepts only RS256-signed Keycloak access tokens.
         *
         * Do not trust an arbitrary algorithm merely because the incoming
         * token declares it in its header.
         */
        if header.alg != Algorithm::RS256 {
            bail!("unsupported JWT signing algorithm: {:?}", header.alg);
        }

        let jwk = jwks
            .find(kid)
            .context("no matching Keycloak signing key found")?;

        /*
         * The selected Keycloak key must contain RSA key material because ORBI
         * currently accepts only RS256-signed access tokens.
         */
        if !matches!(&jwk.algorithm, AlgorithmParameters::RSA(_)) {
            bail!("Keycloak signing key is not an RSA key");
        }

        /*
         * The JWK "alg" field is optional.
         *
         * If Keycloak publishes it, it must agree with ORBI's configured signing
         * policy. If it is absent, signature verification is still constrained to
         * RS256 by Validation below.
         */
        if let Some(key_algorithm) = jwk.common.key_algorithm
            && key_algorithm != KeyAlgorithm::RS256
        {
            bail!(
                "Keycloak signing key declares unsupported algorithm: {}",
                key_algorithm
            );
        }

        let decoding_key =
            DecodingKey::from_jwk(jwk).context("failed to build JWT decoding key")?;

        let mut validation = Validation::new(Algorithm::RS256);

        validation.set_audience(&[&self.audience]);
        validation.set_issuer(&[&self.issuer]);

        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

        let token_data = decode::<KeycloakClaims>(token, &decoding_key, &validation)
            .context("Keycloak access token validation failed")?;

        Ok(token_data.claims)
    }
}
