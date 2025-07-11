use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::database::UserDatabase;
use crate::handlers::{
    health::health_check,
    username::get_username_api,
    update_username::update_username_api,
};

pub fn create_app(database: Arc<dyn UserDatabase>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/username/{username}", get(get_username_api))
        .route("/api/username", post(update_username_api))
        .layer(TraceLayer::new_for_http())
        .with_state(database)
}
