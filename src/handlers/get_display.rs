use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::context;
use std::sync::Arc;
use tracing::info;

use crate::errors::AppError;
use crate::router::AppState;
use crate::validation::ValidatedUsername;

/// GET /display/username/{username} - Display component shows username and display name
pub async fn get_display_username(
    State(app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<Html<String>, AppError> {
    info!("Display request for username: {}", username);
    
    // Validate username
    let validated_username = ValidatedUsername::new(username)?;
    
    // Get user data from database
    let user_data = match app_state.database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User not found - still render template but show error
            let html = app_state.template_service.render("display.html", context! {
                username => validated_username.as_str(),
                error => "User not found"
            })?;
            
            return Ok(Html(html));
        }
        Err(e) => {
            return Err(AppError::database_error(format!("Failed to get user: {}", e)));
        }
    };
    
    // Render the display template
    let html = app_state.template_service.render("display.html", context! {
        username => user_data.username,
        display_name => user_data.display_name,
        title => format!("Display - {}", user_data.username)
    })?;
    
    Ok(Html(html))
}
