use axum::{
    extract::{Extension, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::errors::AppError;
use crate::handlers::get_api_username::UsernameResponse;
use crate::router::AppState;
use crate::validation::{sanitize_display_name, ValidatedDisplayName, ValidatedUsername};

#[derive(Debug, Deserialize)]
pub struct UpdateUsernameRequest {
    pub display_name: String,
}

pub async fn post_api_username(
    State(app_state): State<Arc<AppState>>,
    Extension(username): Extension<String>,
    Json(payload): Json<UpdateUsernameRequest>,
) -> Result<Json<UsernameResponse>, AppError> {
    // Validate username from JWT token
    let validated_username = ValidatedUsername::new(username)?;
    
    // Sanitize and validate display name
    let sanitized_display_name = sanitize_display_name(&payload.display_name);
    let validated_display_name = ValidatedDisplayName::new(sanitized_display_name)?;

    match app_state.database.update_user_display_name(validated_username.as_str(), validated_display_name.as_str()).await {
        Ok(()) => {
            tracing::info!("Updated display name for '{}' to '{}'", validated_username, validated_display_name);
            Ok(Json(UsernameResponse {
                username: validated_username.into_string(),
                display_name: validated_display_name.into_string(),
            }))
        }
        Err(e) => {
            tracing::error!("Database error updating user '{}': {}", validated_username, e);
            Err(AppError::database_error(format!("Failed to update user: {}", e)))
        }
    }
}
