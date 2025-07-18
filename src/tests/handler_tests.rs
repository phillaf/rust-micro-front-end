#[cfg(test)]
mod tests {
    use crate::database::mock::MockUserDatabase;
    use crate::router::create_app;
    use crate::template::TemplateService;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt; // for oneshot
    use hyper::body::to_bytes;
    
    async fn setup_test_app() -> axum::Router {
        // Create mock database
        let db = Arc::new(MockUserDatabase::new());
        
        // Create template service with caching disabled for tests
        let template_service = TemplateService::new(false, false).unwrap();
        
        // Create app with mocks
        create_app(db, template_service)
    }
    
    #[tokio::test]
    async fn test_display_username_existing_user() {
        let app = setup_test_app().await;
        
        // Request for existing user (admin is in mock database)
        let request = Request::builder()
            .uri("/display/username/admin")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status
        assert_eq!(response.status(), StatusCode::OK);
        
        // Get body content
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        
        // Check that the body contains the expected user information
        assert!(body_str.contains("Administrator"));
        assert!(body_str.contains("admin"));
    }
    
    #[tokio::test]
    async fn test_display_username_nonexistent_user() {
        let app = setup_test_app().await;
        
        // Request for non-existent user
        let request = Request::builder()
            .uri("/display/username/nonexistent")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status (should still be 200 OK)
        assert_eq!(response.status(), StatusCode::OK);
        
        // Get body content
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        
        // Check that the body contains the error message
        assert!(body_str.contains("User not found"));
    }
    
    #[tokio::test]
    async fn test_display_username_invalid_username() {
        let app = setup_test_app().await;
        
        // Request with invalid username (too short)
        let request = Request::builder()
            .uri("/display/username/a")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status (should be 400 Bad Request)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    
    #[tokio::test]
    async fn test_api_username_endpoint() {
        let app = setup_test_app().await;
        
        // Request for existing user
        let request = Request::builder()
            .uri("/api/username/admin")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status
        assert_eq!(response.status(), StatusCode::OK);
        
        // Get body content
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        
        // Check that the response is valid JSON containing the user info
        assert!(body_str.contains("\"username\":\"admin\""));
        assert!(body_str.contains("\"display_name\":\"Administrator\""));
    }

    #[tokio::test]
    async fn test_static_endpoints() {
        let app = setup_test_app().await;

        // Test robots.txt
        let request = Request::builder()
            .uri("/robots.txt")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        assert!(body_str.contains("User-agent:"));
        
        // Test sitemap.xml
        let request = Request::builder()
            .uri("/sitemap.xml")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        assert!(body_str.contains("<urlset"));
        
        // Test manifest.json
        let request = Request::builder()
            .uri("/manifest.json")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        assert!(body_str.contains("\"name\":"));
    }
    
    #[tokio::test]
    async fn test_edit_endpoint_unauthorized() {
        let app = setup_test_app().await;
        
        // Request edit endpoint without authentication
        let request = Request::builder()
            .uri("/edit")
            .body(Body::empty())
            .unwrap();
        
        // Process request - should fail auth
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status (should be 401 Unauthorized)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    #[tokio::test]
    async fn test_metrics_endpoint() {
        let app = setup_test_app().await;
        
        // Request metrics endpoint
        let request = Request::builder()
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response status
        assert_eq!(response.status(), StatusCode::OK);
        
        // Get body content
        let body_bytes = to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        
        // Verify the metrics output contains basic Prometheus format
        assert!(body_str.contains("# HELP"));
        assert!(body_str.contains("# TYPE"));
    }
}
