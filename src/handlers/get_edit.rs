use axum::{extract::State, response::Html, Extension};
use minijinja::context;
use std::sync::Arc;
use tracing::info;

use crate::errors::AppError;
use crate::middleware::jwt_auth::Claims;
use crate::router::AppState;
use crate::validation::ValidatedUsername;

/// GET /edit - CMS component for editing display names
pub async fn get_edit(
    State(app_state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Html<String>, AppError> {
    let username = &claims.sub;
    info!("CMS request for username: {}", username);

    // Validate username from JWT token
    let validated_username = ValidatedUsername::new(username.clone())?;

    // Get current user data to pre-populate form
    let current_display_name = match app_state.database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => {
            // If user has a display name, use it; otherwise fall back to username
            if user.display_name.is_empty() {
                user.username
            } else {
                user.display_name
            }
        }
        Ok(None) => {
            // If user doesn't exist in database yet, use username as default
            validated_username.as_str().to_string()
        }
        Err(e) => {
            return Err(AppError::database_error(format!("Failed to get user: {}", e)));
        }
    };

    // Render the edit template
    let html = app_state.template_service.render("edit.html", context! {
        username => validated_username.as_str(),
        display_name => current_display_name,
        title => format!("Edit - {}", validated_username),
        description => format!("Content management system to edit display name for user {}", validated_username.as_str()),
        keywords => "edit, cms, content management, display name, profile"
    })?;

    Ok(Html(html))
}
