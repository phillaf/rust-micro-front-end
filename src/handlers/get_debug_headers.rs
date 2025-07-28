use axum::{extract::Request, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::collections::HashMap;

/// GET /debug/headers - Debug endpoint to check what headers are being sent
pub async fn get_debug_headers(request: Request) -> Result<Json<Value>, StatusCode> {
    let mut headers = HashMap::new();

    for (name, value) in request.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Check for JWT token specifically
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let cookies = request
        .headers()
        .get("cookie")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let has_jwt_cookie = cookies.as_ref().map(|c| c.contains("jwt_token=")).unwrap_or(false);

    let has_jwt_js_cookie = cookies.as_ref().map(|c| c.contains("jwt_token_js=")).unwrap_or(false);

    Ok(Json(json!({
        "headers": headers,
        "method": request.method().to_string(),
        "uri": request.uri().to_string(),
        "auth_header": auth_header,
        "cookies": cookies,
        "has_jwt_cookie": has_jwt_cookie,
        "has_jwt_js_cookie": has_jwt_js_cookie
    })))
}
