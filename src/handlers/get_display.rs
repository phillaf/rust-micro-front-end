use axum::{
    extract::{Path, State},
    response::Html,
};
use minijinja::{context, Environment};
use std::sync::Arc;
use tracing::info;

use crate::database::UserDatabase;
use crate::errors::AppError;
use crate::validation::ValidatedUsername;

/// GET /display/username/{username} - Display component shows username and display name
pub async fn get_display_username(
    State(database): State<Arc<dyn UserDatabase>>,
    Path(username): Path<String>,
) -> Result<Html<String>, AppError> {
    info!("Display request for username: {}", username);
    
    // Validate username
    let validated_username = ValidatedUsername::new(username)?;
    
    // Create template environment
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    
    // Get user data from database
    let user_data = match database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            // User not found - still render template but show error
            let template = env.get_template("display.html")?;
            
            let html = template.render(context! {
                username => validated_username.as_str(),
                error => "User not found"
            })?;
            
            return Ok(Html(html));
        }
        Err(e) => {
            return Err(AppError::database_error(format!("Failed to get user: {}", e)));
        }
    };
    
    // Load and render the display template
    let template = env.get_template("display.html")?;
    
    let html = template.render(context! {
        username => user_data.username,
        display_name => user_data.display_name,
        title => format!("Display - {}", user_data.username)
    })?;
    
    Ok(Html(html))
}
