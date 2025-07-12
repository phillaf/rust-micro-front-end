use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::database::UserDatabase;
use crate::handlers::{
    get_api_username::get_api_username,
    get_debug_set_token::get_debug_set_token,
    get_display::get_display_username,
    get_edit::get_edit,
    get_health::get_health,
    post_api_username::post_api_username,
};
use crate::middleware::jwt_auth_middleware;

pub fn create_app(database: Arc<dyn UserDatabase>) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(get_health))
        .route("/api/username/{username}", get(get_api_username))
        // Web components - public access according to requirements
        .route("/display/username/{username}", get(get_display_username))
        // Debug utility for JWT token setup (development only)
        .route("/debug/set-token/{username}", get(get_debug_set_token));

    // Protected routes (JWT authentication required)
    let protected_routes = Router::new()
        .route("/api/username", post(post_api_username))
        .route("/edit", get(get_edit))
        .layer(middleware::from_fn(jwt_auth_middleware));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(database)
}
