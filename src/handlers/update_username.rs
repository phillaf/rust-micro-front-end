use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::database::{validate_display_name, validate_username, UserDatabase};
use crate::handlers::username::UsernameResponse;

#[derive(Debug, Deserialize)]
pub struct UpdateUsernameRequest {
    pub display_name: String,
}

pub async fn update_username_api(
    State(database): State<Arc<dyn UserDatabase>>,
    Extension(username): Extension<String>,
    Json(payload): Json<UpdateUsernameRequest>,
) -> Result<Json<UsernameResponse>, StatusCode> {
    // Username is now extracted directly from the JWT token via the Extension extractor

    if let Err(e) = validate_display_name(&payload.display_name) {
        tracing::warn!("Invalid display name '{}': {}", payload.display_name, e);
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Err(e) = validate_username(&username) {
        tracing::error!("Invalid username from token '{}': {}", username, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match database.update_user_display_name(&username, &payload.display_name).await {
        Ok(()) => {
            tracing::info!("Updated display name for '{}' to '{}'", username, payload.display_name);
            Ok(Json(UsernameResponse {
                username: username.to_string(),
                display_name: payload.display_name,
            }))
        }
        Err(e) => {
            tracing::error!("Database error updating user '{}': {}", username, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
