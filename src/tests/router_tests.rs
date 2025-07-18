#[cfg(test)]
mod tests {
    use crate::database::mock::MockUserDatabase;
    use crate::router::AppState;
    use crate::template::TemplateService;
    use crate::metrics::AppMetrics;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt; // for oneshot
    
    #[tokio::test]
    async fn test_health_endpoint() {
        // Create mock database
        let db = Arc::new(MockUserDatabase::new());
        
        // Create template service with caching disabled for tests
        let template_service = TemplateService::new(false, false).unwrap();
        
        // For tests, we'll initialize the app state directly to avoid
        // creating new Prometheus metrics (which would fail on subsequent test runs)
        let app_state = Arc::new(AppState {
            database: db.clone(),
            template_service: template_service.clone(),
            // Use the test-specific metrics implementation
            metrics: AppMetrics::new_for_tests(),
        });
        
        // Create a simplified test router
        let app = axum::Router::new()
            .route("/health", axum::routing::get(crate::handlers::get_health::get_health))
            .with_state(app_state);
        
        // Create request to /health
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        
        // Process request
        let response = app.oneshot(request).await.unwrap();
        
        // Check response
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // More route tests would follow a similar pattern
}
