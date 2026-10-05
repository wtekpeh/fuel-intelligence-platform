use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::routes::AppState;

/// Authenticate a human API request using a Keycloak Bearer access token.
///
/// This middleware establishes identity only. ORBI-specific roles,
/// permissions, and organization scope are enforced separately.
pub async fn require_authentication(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    /*
     * Once this middleware is attached to a route, authentication must fail
     * closed. Missing runtime auth state must never turn into an authentication
     * bypass.
     */
    let auth = app_state
        .auth
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let authorization = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let authorization = authorization
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = authorization
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    /*
     * Normal verification is entirely local against the cached Keycloak
     * public keys. No network request to Keycloak occurs here.
     */
    let jwks = auth.jwks.read().await;

    let claims = auth
        .verifier
        .verify(token, &jwks)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    drop(jwks);

    /*
     * Make the authenticated identity available to later middleware and
     * handlers without decoding the JWT again.
     */
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
