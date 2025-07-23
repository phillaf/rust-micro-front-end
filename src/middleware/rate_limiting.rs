use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::warn;

/// Rate limiter based on IP address
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut requests = match self.requests.write() {
            Ok(requests) => requests,
            Err(_) => return false, // Allow request if lock is poisoned
        };

        let ip_requests = requests.entry(ip).or_insert_with(Vec::new);

        // Remove old requests
        ip_requests.retain(|&request_time| request_time > cutoff);

        // Check if under limit
        if ip_requests.len() < self.max_requests {
            ip_requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Rate limiting middleware
pub async fn rate_limiting_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get client IP from headers or connection info
    let client_ip = get_client_ip(&request).unwrap_or(IpAddr::from([127, 0, 0, 1]));

    // Create rate limiter (in production, this would be a singleton)
    let max_requests = std::env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let rate_limiter = RateLimiter::new(max_requests, Duration::from_secs(60));

    if !rate_limiter.check_rate_limit(client_ip) {
        warn!("Rate limit exceeded for IP: {}", client_ip);
        let mut response = Response::new(Body::from("Rate limit exceeded"));
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        response.headers_mut().insert("Retry-After", HeaderValue::from_static("60"));
        return Ok(response);
    }

    let response = next.run(request).await;
    Ok(response)
}

/// Extract client IP from request headers or connection info
fn get_client_ip(request: &Request) -> Option<IpAddr> {
    // Check common proxy headers
    for header_name in ["x-forwarded-for", "x-real-ip", "cf-connecting-ip"] {
        if let Some(header_value) = request.headers().get(header_name) {
            if let Ok(header_str) = header_value.to_str() {
                // Take the first IP if there are multiple (comma-separated)
                if let Some(ip_str) = header_str.split(',').next() {
                    if let Ok(ip) = ip_str.trim().parse() {
                        return Some(ip);
                    }
                }
            }
        }
    }

    // Fallback to connection info (if available)
    // Note: This would require access to connection info which isn't directly available in axum middleware
    // In practice, you'd use a different approach or configure your reverse proxy to set headers
    None
}
