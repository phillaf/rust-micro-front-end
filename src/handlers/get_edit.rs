use axum::{
    extract::State,
    response::Html,
    Extension,
};
use minijinja::{context, Environment};
use std::sync::Arc;
use tracing::info;

use crate::database::UserDatabase;
use crate::errors::AppError;
use crate::validation::ValidatedUsername;

/// GET /edit - CMS component for editing display names
pub async fn get_edit(
    State(database): State<Arc<dyn UserDatabase>>,
    Extension(username): Extension<String>,
) -> Result<Html<String>, AppError> {
    info!("CMS request for username: {}", username);
    
    // Validate username from JWT token
    let validated_username = ValidatedUsername::new(username)?;
    
    // Create template environment
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    
    // Get current user data to pre-populate form
    let current_display_name = match database.get_user(validated_username.as_str()).await {
        Ok(Some(user)) => Some(user.display_name),
        Ok(None) => None,
        Err(e) => {
            return Err(AppError::database_error(format!("Failed to get user: {}", e)));
        }
    };
    
    // Load and render the edit template
    let template = env.get_template("edit.html")?;
    
    let html = template.render(context! {
        username => validated_username.as_str(),
        display_name => current_display_name,
        title => format!("Edit - {}", validated_username)
    })?;
    
    Ok(Html(html))
}
