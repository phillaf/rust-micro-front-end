use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::errors::AppError;
use crate::router::AppState;
use crate::validation::ValidatedUsername;

#[derive(Debug, Serialize)]
pub struct UsernameResponse {
    pub username: String,
    pub display_name: String,
}

pub async fn get_api_username(
    State(app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Json<UsernameResponse>, AppError> {
    let validated_username = ValidatedUsername::new(username)?;

    match app_state.database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => {
            tracing::info!("Retrieved user data for '{}'", validated_username);
            Ok(Json(UsernameResponse {
                username: user.username,
                display_name: user.display_name,
            }))
        }
        Ok(None) => {
            tracing::info!("User '{}' not found", validated_username);
            Err(AppError::user_not_found(validated_username.as_str()))
        }
        Err(e) => {
            tracing::error!("Database error retrieving user '{}': {}", validated_username, e);
            Err(AppError::database_error(format!("Failed to get user: {}", e)))
        }
    }
}
