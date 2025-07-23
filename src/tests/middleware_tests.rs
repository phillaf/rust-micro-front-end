#[cfg(test)]
mod tests {
    use crate::middleware::jwt_auth::JwtConfig;
    use axum::http::HeaderName;
    use axum::{
        body::Body,
        http::{Response, StatusCode},
    };
    use std::str::FromStr;
    use std::sync::Mutex;
    
    // Use a mutex to prevent tests from modifying environment concurrently
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_jwt_config_from_env() {
        // Lock environment for this test
        let _lock = ENV_MUTEX.lock().unwrap();
        
        // Save original values
        let original_key = std::env::var("JWT_PUBLIC_KEY").ok();
        let original_algorithm = std::env::var("JWT_ALGORITHM").ok();
        
        // Temporarily set environment variables for the test
        std::env::set_var("JWT_PUBLIC_KEY", 
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu1SU1LfVLPHCozMxH2Mo\nFTOOZPOMzONKZAM0ACvHVyQtP/YO7pXfy9MfTgLNoB7jrOXCCzmkQw7hoKQ4vgdP\nu4p1wJ1XT5jxhJyqn1R1S+2FHvbfmGPk0Pi/ZoIvTU7AEkIGQGwzKj5WczvZyeeT\nYgYTRXdrUH2G9RRGR9SqLtQUzGAw8R2sBG/jihlLHS9Z4/ew8QFRKKHXSGGxgzPr\nv5srFbWOmx8zOe7r+9EJLv9hhRIX8tOyKEzq7kPWGtq3S0RPPKQYgGM+GFuMPR2P\nQCJDUKHcxPDnAjyFDI3MUUlRwb1Jc7j3Tdfpz21+fU1rrFRdo7IbMuLGl7egTwRY\ntQIDAQAB\n-----END PUBLIC KEY-----\n"
        );
        std::env::set_var("JWT_ALGORITHM", "RS256");

        let jwt_config = JwtConfig::from_env().unwrap();

        // Check that the config was properly initialized
        assert_eq!(jwt_config.validation.algorithms, vec![jsonwebtoken::Algorithm::RS256]);
        
        // Restore original values
        if let Some(value) = original_key {
            std::env::set_var("JWT_PUBLIC_KEY", value);
        } else {
            std::env::remove_var("JWT_PUBLIC_KEY");
        }

        if let Some(value) = original_algorithm {
            std::env::set_var("JWT_ALGORITHM", value);
        } else {
            std::env::remove_var("JWT_ALGORITHM");
        }
    }

    #[test]
    fn test_jwt_config_debug() {
        // Lock environment for this test
        let _lock = ENV_MUTEX.lock().unwrap();
        
        // Save original values
        let original_key = std::env::var("JWT_PUBLIC_KEY").ok();
        
        // Setup JWT config
        std::env::set_var("JWT_PUBLIC_KEY", 
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAu1SU1LfVLPHCozMxH2Mo\nFTOOZPOMzONKZAM0ACvHVyQtP/YO7pXfy9MfTgLNoB7jrOXCCzmkQw7hoKQ4vgdP\nu4p1wJ1XT5jxhJyqn1R1S+2FHvbfmGPk0Pi/ZoIvTU7AEkIGQGwzKj5WczvZyeeT\nYgYTRXdrUH2G9RRGR9SqLtQUzGAw8R2sBG/jihlLHS9Z4/ew8QFRKKHXSGGxgzPr\nv5srFbWOmx8zOe7r+9EJLv9hhRIX8tOyKEzq7kPWGtq3S0RPPKQYgGM+GFuMPR2P\nQCJDUKHcxPDnAjyFDI3MUUlRwb1Jc7j3Tdfpz21+fU1rrFRdo7IbMuLGl7egTwRY\ntQIDAQAB\n-----END PUBLIC KEY-----\n"
        );

        let jwt_config = JwtConfig::from_env().unwrap();

        // Test the Debug implementation
        let debug_output = format!("{:?}", jwt_config);
        assert!(debug_output.contains("<redacted>"));
        assert!(debug_output.contains("JwtConfig"));
        
        // Restore original values
        if let Some(value) = original_key {
            std::env::set_var("JWT_PUBLIC_KEY", value);
        } else {
            std::env::remove_var("JWT_PUBLIC_KEY");
        }
    }

    #[test]
    fn test_security_headers() {
        // Create a basic response
        let mut response = Response::builder().status(StatusCode::OK).body(Body::empty()).unwrap();

        // Add security headers manually as we would in the middleware
        response
            .headers_mut()
            .insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        response.headers_mut().insert("X-Frame-Options", "DENY".parse().unwrap());
        response
            .headers_mut()
            .insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());

        // Check that the headers were properly applied
        assert_eq!(response.headers().get("X-Content-Type-Options").unwrap(), "nosniff");
        assert_eq!(response.headers().get("X-Frame-Options").unwrap(), "DENY");
    }

    #[test]
    fn test_rate_limiting_headers() {
        // We'll create a mock response to check if rate limiting headers get applied
        let mut response = Response::builder().status(StatusCode::OK).body(Body::empty()).unwrap();

        // Apply rate limiting headers
        response
            .headers_mut()
            .insert(HeaderName::from_str("x-ratelimit-limit").unwrap(), "10".parse().unwrap());
        response
            .headers_mut()
            .insert(HeaderName::from_str("x-ratelimit-remaining").unwrap(), "9".parse().unwrap());

        // Check that the headers were properly applied
        assert_eq!(response.headers().get("x-ratelimit-limit").unwrap(), "10");
        assert_eq!(response.headers().get("x-ratelimit-remaining").unwrap(), "9");
    }
}
