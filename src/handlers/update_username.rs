use axum::{
    extract::State,
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
    Json(request): Json<UpdateUsernameRequest>,
) -> Result<Json<UsernameResponse>, StatusCode> {
    // TODO: Extract username from JWT token - for now using hardcoded username
    let username = "testuser";
    
    if let Err(e) = validate_display_name(&request.display_name) {
        tracing::warn!("Invalid display name '{}': {}", request.display_name, e);
        return Err(StatusCode::BAD_REQUEST);
    }

    if let Err(e) = validate_username(username) {
        tracing::error!("Invalid username from token '{}': {}", username, e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match database.update_user_display_name(username, &request.display_name).await {
        Ok(()) => {
            tracing::info!("Updated display name for '{}' to '{}'", username, request.display_name);
            Ok(Json(UsernameResponse {
                username: username.to_string(),
                display_name: request.display_name,
            }))
        }
        Err(e) => {
            tracing::error!("Database error updating user '{}': {}", username, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
