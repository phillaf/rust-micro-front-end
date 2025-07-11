use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::database::{validate_username, UserDatabase};

#[derive(Debug, Serialize)]
pub struct UsernameResponse {
    pub username: String,
    pub display_name: String,
}

pub async fn get_username_api(
    State(database): State<Arc<dyn UserDatabase>>,
    Path(username): Path<String>,
) -> Result<Json<UsernameResponse>, StatusCode> {
    if let Err(e) = validate_username(&username) {
        tracing::warn!("Invalid username format '{}': {}", username, e);
        return Err(StatusCode::BAD_REQUEST);
    }

    match database.get_user(&username).await {
        Ok(Some(user)) => {
            tracing::info!("Retrieved user data for '{}'", username);
            Ok(Json(UsernameResponse {
                username: user.username,
                display_name: user.display_name,
            }))
        }
        Ok(None) => {
            tracing::info!("User '{}' not found", username);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Database error retrieving user '{}': {}", username, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
