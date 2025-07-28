pub mod jwt_auth;
pub mod rate_limiting;
pub mod security;

pub use jwt_auth::*;
pub use rate_limiting::*;
pub use security::*;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::warn;

use crate::{
    metrics::{track_auth_failure, track_auth_success},
    router::AppState,
};

/// Middleware to track authentication metrics
pub async fn auth_metrics_middleware(State(app_state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    // Check response status to determine authentication success/failure
    match response.status() {
        StatusCode::UNAUTHORIZED => {
            warn!("Authentication failed");
            track_auth_failure(&app_state.metrics, "invalid_token");
        }
        StatusCode::FORBIDDEN => {
            warn!("Authorization failed");
            track_auth_failure(&app_state.metrics, "insufficient_permissions");
        }
        _ => {
            // Check if we have a username in the extensions (successful auth)
            if let Some(username) = response.extensions().get::<String>() {
                track_auth_success(&app_state.metrics, username);
            }
        }
    }

    response
}
