use axum::{
    http::{header, Method},
    middleware,
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Duration};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

use crate::database::UserDatabase;
use crate::handlers::{
    get_api_username::get_api_username,
    get_debug_set_token::get_debug_set_token,
    get_display::get_display_username,
    get_edit::get_edit,
    get_health::get_health,
    post_api_username::post_api_username,
};
use crate::middleware::{jwt_auth_middleware, rate_limiting_middleware, security_headers_middleware};
use crate::template::TemplateService;

/// Application state containing all shared services
#[derive(Clone)]
pub struct AppState {
    pub database: Arc<dyn UserDatabase>,
    pub template_service: TemplateService,
}

pub fn create_app(database: Arc<dyn UserDatabase>, template_service: TemplateService) -> Router {
    let app_state = Arc::new(AppState {
        database,
        template_service,
    });

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(get_health))
        .route("/api/username/{username}", get(get_api_username))
        .route("/display/username/{username}", get(get_display_username))
        .route("/debug/set-token/{username}", get(get_debug_set_token));

    // Protected routes (JWT authentication required) - apply rate limiting to auth endpoints
    let protected_routes = Router::new()
        .route("/api/username", post(post_api_username))
        .route("/edit", get(get_edit))
        .layer(middleware::from_fn(rate_limiting_middleware))
        .layer(middleware::from_fn(jwt_auth_middleware));

    // Combine routes with performance and security optimizations
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new().gzip(true).br(true))
        .layer(RequestBodyLimitLayer::new(1024 * 16)) // 16KB limit
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                .max_age(Duration::from_secs(3600)),
        )
        .with_state(app_state)
}
