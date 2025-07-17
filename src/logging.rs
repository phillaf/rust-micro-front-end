use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Structure for tracking request context and correlation IDs
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestContext {
    /// Unique ID for this request (correlation ID)
    pub request_id: String,
    
    /// Username if authenticated
    pub username: Option<String>,
    
    /// Start time for this request
    pub start_time: Instant,
    
    /// Request path
    pub path: String,
    
    /// Request method
    pub method: String,
    
    /// User agent
    pub user_agent: Option<String>,
}

#[allow(dead_code)]
impl RequestContext {
    /// Create a new request context with a unique correlation ID
    pub fn new(path: String, method: String, user_agent: Option<String>) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            username: None,
            start_time: Instant::now(),
            path,
            method,
            user_agent,
        }
    }
    
    /// Set the authenticated username
    pub fn with_username(mut self, username: Option<String>) -> Self {
        self.username = username;
        self
    }
    
    /// Log request start with context
    pub fn log_request_start(&self) {
        info!(
            request_id = %self.request_id,
            path = %self.path,
            method = %self.method,
            username = ?self.username,
            user_agent = ?self.user_agent,
            "Request started"
        );
    }
    
    /// Log request completion with duration
    pub fn log_request_completion(&self, status_code: u16) {
        let duration = self.start_time.elapsed();
        
        info!(
            request_id = %self.request_id,
            path = %self.path,
            method = %self.method,
            username = ?self.username,
            status = status_code,
            duration_ms = duration.as_millis() as u64,
            "Request completed"
        );
    }
    
    /// Log a database operation with context
    pub fn log_database_operation(&self, operation: &str, success: bool, duration: std::time::Duration) {
        if success {
            debug!(
                request_id = %self.request_id,
                operation = %operation,
                duration_ms = duration.as_millis() as u64,
                "Database operation completed successfully"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                operation = %operation,
                duration_ms = duration.as_millis() as u64,
                "Database operation failed"
            );
        }
    }
    
    /// Log an authentication event with context
    pub fn log_auth_event(&self, success: bool, reason: Option<&str>) {
        if success {
            info!(
                request_id = %self.request_id,
                username = ?self.username,
                "Authentication successful"
            );
        } else {
            warn!(
                request_id = %self.request_id,
                username = ?self.username,
                reason = reason,
                "Authentication failed"
            );
        }
    }
    
    /// Log an error with context
    pub fn log_error(&self, error: &anyhow::Error) {
        error!(
            request_id = %self.request_id,
            path = %self.path,
            method = %self.method,
            username = ?self.username,
            error = %error,
            "Request error occurred"
        );
    }
}

/// Create a middleware for request context logging and tracing with correlation IDs
pub async fn request_context_middleware(
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract basic request information
    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    
    // Extract optional user agent
    let user_agent = req
        .headers()
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Create request context with correlation ID
    let request_ctx = RequestContext::new(path, method, user_agent);
    
    // Log request start
    request_ctx.log_request_start();
    
    // Insert request_id as a header for correlation across services
    if let Ok(header_value) = axum::http::header::HeaderValue::from_str(&request_ctx.request_id) {
        req.headers_mut().insert("X-Request-ID", header_value);
    }
    
    // Process the request
    let start = Instant::now();
    let mut response = next.run(req).await;
    let duration = start.elapsed();
    
    // Add correlation ID to response headers
    if let Ok(header_value) = axum::http::header::HeaderValue::from_str(&request_ctx.request_id) {
        response.headers_mut().insert("X-Request-ID", header_value);
    }
    
    // Log request completion with status code
    let status = response.status().as_u16();
    request_ctx.log_request_completion(status);
    
    // If it's an error response (4xx or 5xx), log more details
    if status >= 400 {
        let level = if status < 500 { "warn" } else { "error" };
        
        warn!(
            request_id = %request_ctx.request_id,
            status_code = status,
            path = %request_ctx.path,
            method = %request_ctx.method,
            duration_ms = duration.as_millis() as u64,
            level = level,
            "Request resulted in error response"
        );
    }
    
    response
}

/// Create a middleware for capturing and logging errors with context
pub async fn error_logging_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract request ID from headers if it exists
    let request_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
        
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();
    
    // Process the request
    let response = next.run(req).await;
    
    // Check if it's an error response
    let status = response.status();
    if status.is_server_error() {
        // Log server errors with detailed context
        error!(
            request_id = %request_id,
            status_code = %status.as_u16(),
            path = %path,
            method = %method,
            "Server error occurred"
        );
    } else if status.is_client_error() {
        // Log client errors
        warn!(
            request_id = %request_id,
            status_code = %status.as_u16(),
            path = %path,
            method = %method,
            "Client error occurred"
        );
    }
    
    response
}

/// Create a middleware for tracking and logging security events
pub async fn security_event_logging_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Extract request ID from headers if it exists
    let request_id = req
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
        
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();
    
    // Check if it's a security-sensitive endpoint
    let is_auth_endpoint = path.contains("/api/username") || 
                          path.contains("/debug/set-token") || 
                          path.contains("/edit");
                          
    if is_auth_endpoint {
        // Log access to security-sensitive endpoints
        info!(
            request_id = %request_id,
            path = %path,
            method = %method,
            "Access to security-sensitive endpoint"
        );
    }
    
    // Process the request
    let response = next.run(req).await;
    
    // Check for authentication failures (401)
    if response.status() == axum::http::StatusCode::UNAUTHORIZED {
        warn!(
            request_id = %request_id,
            path = %path,
            method = %method,
            "Authentication failure"
        );
    }
    
    // Check for authorization failures (403)
    if response.status() == axum::http::StatusCode::FORBIDDEN {
        warn!(
            request_id = %request_id,
            path = %path,
            method = %method,
            "Authorization failure"
        );
    }
    
    response
}
