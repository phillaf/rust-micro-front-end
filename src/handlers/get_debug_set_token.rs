use axum::{
    extract::{Path, Query, State},
    http::{header::SET_COOKIE, StatusCode},
    response::Response,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::middleware::jwt_auth::Claims;
use crate::router::AppState;

/// GET /debug/set-token/{username} - Debug utility to set JWT token in browser
pub async fn get_debug_set_token(
    State(_app_state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, StatusCode> {
    info!("Debug: Setting JWT token for username: {}", username);

    // Use token from query parameter if provided, otherwise generate one
    let (token, expiration_time) = if let Some(provided_token) = params.get("token") {
        let token = provided_token.clone();
        // Parse the provided token to extract expiration
        let exp = extract_token_expiration(&token).unwrap_or_else(|_| {
            // If we can't extract expiration, default to 1 hour from now
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize;
            now + 3600
        });
        (token, exp)
    } else {
        info!("No token provided, generating new debug JWT");
        let (new_token, exp) = generate_debug_jwt(&username)?;
        (new_token, exp)
    };

    tracing::debug!("Generated/provided token length: {}", token.len());

    // Calculate max age in seconds (how long until token expires)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let max_age = if expiration_time > now {
        expiration_time - now
    } else {
        // If token is already expired, set a minimal cookie lifetime
        // This shouldn't normally happen but handles edge cases
        60 // 1 minute
    };

    // Format cookie expiration for display
    let expiration_minutes = max_age / 60;
    let expiration_seconds = max_age % 60;
    let expiration_text = if expiration_minutes > 0 {
        format!("{} minutes {} seconds", expiration_minutes, expiration_seconds)
    } else {
        format!("{} seconds", expiration_seconds)
    };

    // Create HTML content for the response
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>JWT Token Set Successfully</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            background: #f5f5f5;
            color: #333;
            line-height: 1.6;
        }}
        .container {{
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .success {{
            color: #106330;
            background: #d5f4e6;
            padding: 15px;
            border-radius: 4px;
            margin-bottom: 20px;
            border: 1px solid #106330;
        }}
        .btn {{
            background: #1a6ea5;
            color: white;
            padding: 10px 20px;
            text-decoration: none;
            border-radius: 4px;
            display: inline-block;
            margin: 10px 5px 0 0;
        }}
        .btn:hover {{
            background: #155a87;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔑 JWT Token Set Successfully</h1>
        <div class="success">
            ✅ JWT token has been set for user: <strong>{username}</strong>
        </div>
        <p>The JWT token has been stored in your browser cookies and is ready to use.</p>
        <div>
            <a href="/edit" class="btn">Test Protected Page (/edit)</a>
            <a href="/api/username" class="btn">Test API Endpoint</a>
            <a href="/display" class="btn">View Display Page</a>
        </div>
        <p><small>Token will expire in {expiration_text}. Token length: {token_len} characters.</small></p>
    </div>
</body>
</html>"#,
        username = username,
        token_len = token.len(),
        expiration_text = expiration_text
    ); // Build response with a single Set-Cookie header and HTML content
       // Include Max-Age to ensure the cookie expires when the token expires
       // Use HttpOnly=false to allow JavaScript access
    let cookie = format!("jwt_token={}; Path=/; SameSite=Lax; Max-Age={}; HttpOnly=false", token, max_age);

    info!("Setting JWT cookie for user: {} with Max-Age={}", username, max_age);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .header(SET_COOKIE, &cookie)
        .body(axum::body::Body::from(html_content))
        .map_err(|_| {
            tracing::error!("Failed to build response");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Set jwt_token cookie with proper expiration");

    Ok(response)
}

// Function to extract expiration time from a JWT token without validating signature
fn extract_token_expiration(token: &str) -> Result<usize, StatusCode> {
    // Split the token into parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Decode the payload (middle part)
    let payload = URL_SAFE_NO_PAD.decode(parts[1]).map_err(|_| {
        tracing::error!("Failed to decode JWT payload");
        StatusCode::BAD_REQUEST
    })?;

    // Parse the payload as JSON
    let payload_str = String::from_utf8(payload).map_err(|_| {
        tracing::error!("Failed to convert JWT payload to string");
        StatusCode::BAD_REQUEST
    })?;

    let json: serde_json::Value = serde_json::from_str(&payload_str).map_err(|_| {
        tracing::error!("Failed to parse JWT payload as JSON");
        StatusCode::BAD_REQUEST
    })?;

    // Extract the expiration time
    let exp = json.get("exp").and_then(|e| e.as_u64()).ok_or_else(|| {
        tracing::error!("No expiration found in JWT");
        StatusCode::BAD_REQUEST
    })? as usize;

    Ok(exp)
}

fn generate_debug_jwt(username: &str) -> Result<(String, usize), StatusCode> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Read private key from environment
    let private_key_pem = std::env::var("JWT_PRIVATE_KEY").map_err(|_| {
        tracing::error!("JWT_PRIVATE_KEY environment variable not set");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).map_err(|e| {
        tracing::error!("Failed to parse JWT private key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;

    // Set token expiration to 1 hour (3600 seconds) from now
    let expiration = now + 3600;

    let claims = Claims {
        sub: username.to_string(),
        iat: now,
        exp: expiration, // 1 hour expiry
        aud: "micro-frontend-service".to_string(),
        iss: "test-auth-service".to_string(),
    };

    let header = Header::new(Algorithm::RS256);
    let token = encode(&header, &claims, &encoding_key).map_err(|e| {
        tracing::error!("Failed to encode JWT: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((token, expiration))
}
