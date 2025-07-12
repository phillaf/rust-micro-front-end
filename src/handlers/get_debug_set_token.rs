use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use minijinja::Environment;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::database::UserDatabase;

/// GET /debug/set-token/{username} - Debug utility to set JWT token in browser
pub async fn get_debug_set_token(
    State(_database): State<Arc<dyn UserDatabase>>,
    Path(username): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, StatusCode> {
    info!("Debug: Setting JWT token for username: {}", username);
    
    // Use token from query parameter if provided, otherwise generate one
    let token = if let Some(provided_token) = params.get("token") {
        provided_token.clone()
    } else {
        generate_debug_jwt(&username)?
    };
    
    // Create template environment
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    
    let template_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>JWT Token Debug Helper</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .success {
            color: #27ae60;
            background: #d5f4e6;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            border: 1px solid #27ae60;
        }
        .token {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 4px;
            font-family: monospace;
            word-break: break-all;
            margin: 15px 0;
            border: 1px solid #dee2e6;
        }
        .btn {
            background: #3498db;
            color: white;
            padding: 12px 24px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            margin: 10px 10px 10px 0;
        }
        .btn:hover {
            background: #2980b9;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>JWT Token Set Successfully!</h1>
        
        <div class="success">
            ✓ JWT token has been automatically set for user: <strong>{{ username }}</strong>
        </div>
        
        <p>The token has been stored and will be automatically used by the CMS form.</p>
        
        <div class="token">
            <strong>Token:</strong><br>
            {{ token }}
        </div>
        
        <p><strong>Token expires in 1 hour</strong></p>
        
        <div>
            <a href="/edit" class="btn">Go to CMS (Edit)</a>
            <a href="/display/username/{{ username }}" class="btn">View Display</a>
        </div>
        
        <hr style="margin: 30px 0;">
        
        <h3>Debug Information</h3>
        <p>This is a development-only utility. The token is automatically injected via JavaScript.</p>
        <p>To test with a different user, visit: <code>/debug/set-token/&lt;username&gt;</code></p>
    </div>
    
    <script>
        // Automatically set the JWT token
        const token = "{{ token }}";
        
        // Try multiple storage methods
        try {
            // First try sessionStorage
            if (typeof(Storage) !== "undefined" && sessionStorage) {
                sessionStorage.setItem('jwt_token', token);
                console.log('Token stored in sessionStorage');
            }
        } catch(e) {
            console.log('sessionStorage not available:', e.message);
        }
        
        try {
            // Then try localStorage as fallback
            if (typeof(Storage) !== "undefined" && localStorage) {
                localStorage.setItem('jwt_token', token);
                console.log('Token stored in localStorage');
            }
        } catch(e) {
            console.log('localStorage not available:', e.message);
        }
        
        // As final fallback, store in window object
        window.jwt_token = token;
        console.log('Token stored in window.jwt_token');
        
        // Also set it in a cookie as ultimate fallback
        document.cookie = `jwt_token=${token}; path=/; max-age=3600; SameSite=Lax`;
        console.log('Token stored in cookie');
        
        console.log('JWT token set for user: {{ username }}');
        console.log('Token: ' + token);
    </script>
</body>
</html>
"#;
    
    let html = template_content
        .replace("{{ username }}", &username)
        .replace("{{ token }}", &token);
    
    Ok(Html(html))
}

fn generate_debug_jwt(username: &str) -> Result<String, StatusCode> {
    use std::process::Command;
    
    // Generate JWT token using the shell script
    let output = Command::new("bash")
        .arg("scripts/jwt_test_helper.sh")
        .arg(username)
        .output()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !output.status.success() {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let output_str = String::from_utf8(output.stdout)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Extract just the token from the script output
    // The script outputs "Token (copy this):" followed by the token
    for line in output_str.lines() {
        let line = line.trim();
        if line.starts_with("eyJ") && line.contains('.') {
            return Ok(line.to_string());
        }
    }
    
    Err(StatusCode::INTERNAL_SERVER_ERROR)
}
