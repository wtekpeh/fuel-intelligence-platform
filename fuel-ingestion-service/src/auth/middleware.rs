use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

use crate::models::OrbiUser;
use axum::Extension;

use crate::{repository::find_orbi_user_by_keycloak_subject, routes::AppState};

/// Authenticate a human API request and resolve its ORBI identity.
///
/// Keycloak verifies who the user is.
/// ORBI determines whether that identity has an active application account.
///
/// Roles, permissions, and organization scope are enforced separately.
pub async fn require_authentication(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    /*
     * Authentication must fail closed.
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
     * Verify the JWT locally using cached Keycloak public keys.
     */
    let jwks = auth.jwks.read().await;

    let claims = auth
        .verifier
        .verify(token, &jwks)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    drop(jwks);

    /*
     * Resolve the authenticated Keycloak subject to an ORBI user.
     *
     * A valid Keycloak account alone does not grant ORBI access.
     */
    let orbi_user = find_orbi_user_by_keycloak_subject(&app_state.db_pool, &claims.sub)
        .await
        .map_err(|error| {
            eprintln!("Failed to resolve ORBI user: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::FORBIDDEN)?;

    /*
     * Disabled ORBI accounts cannot access protected APIs,
     * even when their Keycloak token remains valid.
     */
    if !orbi_user.is_active {
        return Err(StatusCode::FORBIDDEN);
    }

    /*
     * Make both identities available to downstream middleware and handlers.
     *
     * KeycloakClaims: verified external identity.
     * OrbiUser: resolved internal application identity.
     */
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(orbi_user);

    Ok(next.run(request).await)
}

/// Authenticate a Keycloak identity without requiring an existing ORBI user.
///
/// Used by client onboarding, where the ORBI application account
/// and organization membership have not yet been created.
pub async fn require_keycloak_authentication(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
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

    let jwks = auth.jwks.read().await;

    let claims = auth
        .verifier
        .verify(token, &jwks)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    drop(jwks);

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Authorize internal ORBI hardware-management operations.
///
/// This middleware must run after require_authentication,
/// which resolves the authenticated ORBI user.
pub async fn require_platform_admin(
    Extension(orbi_user): Extension<OrbiUser>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if orbi_user.platform_role.as_deref() != Some("SUPER_ADMIN") {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    fn test_user(platform_role: Option<&str>) -> OrbiUser {
        OrbiUser {
            id: Uuid::new_v4(),
            keycloak_subject: "test-keycloak-subject".to_string(),
            username: Some("test-user".to_string()),
            email: Some("test@example.com".to_string()),
            platform_role: platform_role.map(str::to_string),
            is_active: true,
        }
    }

    async fn check_access(user: OrbiUser) -> StatusCode {
        let app = Router::new()
            .route("/api/platform/me", get(|| async { StatusCode::OK }))
            .route_layer(middleware::from_fn(require_platform_admin))
            .layer(Extension(user));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/platform/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        response.status()
    }

    #[tokio::test]
    async fn super_admin_can_access_management() {
        let status = check_access(test_user(Some("SUPER_ADMIN"))).await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn non_admin_platform_user_is_forbidden() {
        let status = check_access(test_user(Some("PLATFORM_USER"))).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn customer_without_platform_role_is_forbidden() {
        let status = check_access(test_user(None)).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}
