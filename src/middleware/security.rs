use axum::{
    body::Body,
    http::{header, HeaderValue, Request, Response, StatusCode},
    middleware::Next,
};

pub async fn security_headers_middleware(request: Request<Body>, next: Next) -> Result<Response<Body>, StatusCode> {
    let uri_path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Security headers for Lighthouse 100/100 and security best practices
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
    headers.insert("Referrer-Policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    // Strict Content Security Policy for inline scripts (as required by architecture)
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'",
        ),
    );

    // Cache control headers for performance
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        if let Ok(content_type_str) = content_type.to_str() {
            if content_type_str.starts_with("text/html") {
                // Check if this is a user-specific page that should not be cached
                if is_user_specific_page(&uri_path) {
                    // User-specific pages - no cache
                    headers.insert(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
                    );
                    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
                    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
                } else {
                    // Public HTML pages - cache but revalidate
                    headers.insert(
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=300, must-revalidate"),
                    );
                }
            } else if content_type_str.starts_with("text/css")
                || content_type_str.starts_with("application/javascript")
                || content_type_str.starts_with("image/")
            {
                // Static assets - longer cache
                headers.insert(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                );
            }
        }
    }

    Ok(response)
}

/// Helper function to determine if a page contains user-specific content that shouldn't be cached
fn is_user_specific_page(path: &str) -> bool {
    path == "/edit" || path.starts_with("/display/username/")
}
