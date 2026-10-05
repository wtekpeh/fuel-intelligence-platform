//! Authentication and authorization infrastructure.
//!
//! This module owns human-user security for ORBI's HTTP API.
//! Keycloak will provide user authentication and identity, while ORBI
//! remains responsible for application authorization and organization scope.
//!
//! Device-to-platform authentication is intentionally separate from this
//! module because physical ORBI hardware does not authenticate as a human
//! Keycloak user.

pub mod claims;
pub mod keycloak;
pub mod middleware;
pub mod state;
pub mod verifier;
