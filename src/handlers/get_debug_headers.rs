use axum::{
    extract::Request,
    http::StatusCode,
    response::Json,
};
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
    
    Ok(Json(json!({
        "headers": headers,
        "method": request.method().to_string(),
        "uri": request.uri().to_string()
    })))
}
