#[cfg(test)]
mod tests {
    use crate::database::mock::MockUserDatabase;
    use crate::router::create_app;
    use crate::template::TemplateService;
    use axum::body::Body;
    use axum::http::{header, Request, StatusCode};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::fs;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt; // for oneshot

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String, // Subject (username)
        iat: usize,  // Issued at
        exp: usize,  // Expiration time
        aud: String, // Audience
        iss: String, // Issuer
    }

    /// Generate a real JWT token for testing
    fn generate_test_jwt(username: &str) -> String {
        // Try to load the private key from the test file
        let private_key =
            fs::read_to_string("scripts/jwt_private_key.pem").expect("Failed to read JWT private key file");

        let encoding_key = EncodingKey::from_rsa_pem(private_key.as_bytes()).expect("Failed to parse JWT private key");

        // Create claims
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as usize;

        let claims = Claims {
            sub: username.to_string(),
            iat: now,
            exp: now + 3600, // Valid for 1 hour
            aud: "micro-frontend-service".to_string(),
            iss: "test-auth-service".to_string(),
        };

        // Generate token
        let token =
            encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).expect("Failed to generate JWT token");

        format!("Bearer {}", token)
    }

    async fn setup_test_app() -> axum::Router {
        // Create mock database
        let db = Arc::new(MockUserDatabase::new());

        // Create template service
        let template_service = TemplateService::new(true, false).unwrap();

        // Use a mutex to prevent environment variable race conditions
        static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _lock = ENV_MUTEX.lock().unwrap();

        // Store original env vars to restore later (if needed)
        let _original_jwt_key = env::var("JWT_PUBLIC_KEY").ok();

        // Generate fresh test keys
        let output = std::process::Command::new("./scripts/generate_jwt_keys.sh")
            .output()
            .expect("Failed to execute generate_jwt_keys.sh");

        println!("Key generation output: {}", String::from_utf8_lossy(&output.stdout));

        // Read the JWT keys for test
        let public_key = fs::read_to_string("scripts/jwt_public_key.pem").expect("Failed to read JWT public key file");

        // Verify key content
        assert!(!public_key.is_empty(), "Public key should not be empty");
        assert!(public_key.contains("BEGIN PUBLIC KEY"), "Public key should contain header");
        assert!(public_key.contains("END PUBLIC KEY"), "Public key should contain footer");

        // Set environment variables for JWT validation
        env::set_var("JWT_PUBLIC_KEY", public_key);
        env::set_var("JWT_ALGORITHM", "RS256");
        env::set_var("JWT_AUDIENCE", "micro-frontend-service");
        env::set_var("JWT_ISSUER", "test-auth-service");

        // Create app
        create_app(db, template_service)
    }

    #[tokio::test]
    async fn test_protected_edit_endpoint_with_valid_jwt() {
        let app = setup_test_app().await;

        // Generate valid JWT token
        let auth_token = generate_test_jwt("admin");

        // Create request with JWT token
        let request = Request::builder()
            .uri("/edit")
            .header(header::AUTHORIZATION, auth_token)
            .body(Body::empty())
            .unwrap();

        // Process request
        let response = app.oneshot(request).await.unwrap();

        // Should be authorized and return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        // We avoid reading response body since it's causing issues with compatibility
    }

    #[tokio::test]
    async fn test_protected_edit_endpoint_with_invalid_jwt() {
        let app = setup_test_app().await;

        // Create request with invalid JWT token
        let request = Request::builder()
            .uri("/edit")
            .header(header::AUTHORIZATION, "Bearer invalid.jwt.token")
            .body(Body::empty())
            .unwrap();

        // Process request
        let response = app.oneshot(request).await.unwrap();

        // Should be unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_post_api_username_with_valid_jwt() {
        let app = setup_test_app().await;

        // Generate valid JWT token
        let auth_token = generate_test_jwt("admin");

        // Create POST request with JWT token and JSON body
        let json_body = r#"{"display_name":"Updated Admin Name"}"#;
        let request = Request::builder()
            .uri("/api/username")
            .method("POST")
            .header(header::AUTHORIZATION, auth_token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json_body))
            .unwrap();

        // Process request
        let response = app.oneshot(request).await.unwrap();

        // Should be successful
        assert_eq!(response.status(), StatusCode::OK);

        // We avoid reading response body since it's causing issues with compatibility
    }
}
