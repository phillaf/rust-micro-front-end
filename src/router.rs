use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::database::UserDatabase;
use crate::handlers::{health::health_check, update_username::update_username_api, username::get_username_api};
use crate::middleware::jwt_auth_middleware;

pub fn create_app(database: Arc<dyn UserDatabase>) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/username/{username}", get(get_username_api));

    // Protected routes (JWT authentication required)
    let protected_routes = Router::new()
        .route("/api/username", post(update_username_api))
        .layer(middleware::from_fn(jwt_auth_middleware));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(database)
}
