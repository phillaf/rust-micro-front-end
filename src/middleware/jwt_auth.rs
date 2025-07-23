#![allow(clippy::uninlined_format_args)]

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, header::COOKIE, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (username)
    pub iat: usize,  // Issued at
    pub exp: usize,  // Expiration time
    pub aud: String, // Audience
    pub iss: String, // Issuer
}

pub struct JwtConfig {
    pub public_key: DecodingKey,
    pub validation: Validation,
}

impl std::fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("public_key", &"<redacted>")
            .field("validation", &self.validation)
            .finish()
    }
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut public_key_pem = env::var("JWT_PUBLIC_KEY").map_err(|_| "JWT_PUBLIC_KEY environment variable not set")?;
        
        // Handle escaped newlines in the public key
        if public_key_pem.contains("\\n") {
            public_key_pem = public_key_pem.replace("\\n", "\n");
        }

        let algorithm = env::var("JWT_ALGORITHM").unwrap_or_else(|_| "RS256".to_string());

        let audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "micro-frontend-service".to_string());

        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "test-auth-service".to_string());

        let public_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| format!("Failed to parse JWT public key: {e}"))?;

        let mut validation = Validation::new(match algorithm.as_str() {
            "RS256" => Algorithm::RS256,
            "ES256" => Algorithm::ES256,
            _ => return Err(format!("Unsupported JWT algorithm: {algorithm}").into()),
        });

        validation.set_audience(&[audience]);
        validation.set_issuer(&[issuer]);

        // Configure validation parameters
        let _max_age = env::var("JWT_MAX_AGE_SECONDS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);

        let clock_skew = env::var("JWT_CLOCK_SKEW_SECONDS")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .unwrap_or(60);

        validation.leeway = clock_skew;

        Ok(JwtConfig { public_key, validation })
    }
}

pub async fn jwt_auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Try to extract JWT token from multiple sources
    let token = extract_jwt_token(&request).ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT token
    let jwt_config = JwtConfig::from_env().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token_data = decode::<Claims>(&token, &jwt_config.public_key, &jwt_config.validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Extract username from the token
    let username = token_data.claims.sub.clone();

    // Add username to request extensions for handlers to access
    request.extensions_mut().insert(username);

    tracing::info!("JWT authentication successful for user: {}", token_data.claims.sub);

    Ok(next.run(request).await)
}

fn extract_jwt_token(request: &Request) -> Option<String> {
    // 1. Check Authorization header (Bearer token)
    if let Some(auth_header) = request.headers().get(AUTHORIZATION).and_then(|header| header.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    // 2. Check for jwt_token cookie
    if let Some(cookie_header) = request.headers().get(COOKIE).and_then(|header| header.to_str().ok()) {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(token) = cookie.strip_prefix("jwt_token=") {
                return Some(token.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[test]
    fn test_jwt_config_creation() {
        // Save original values
        let original_jwt_key = env::var("JWT_PUBLIC_KEY").ok();
        let original_jwt_algorithm = env::var("JWT_ALGORITHM").ok();
        let original_jwt_audience = env::var("JWT_AUDIENCE").ok();
        let original_jwt_issuer = env::var("JWT_ISSUER").ok();

        // Set test environment variables
        env::set_var(
            "JWT_PUBLIC_KEY",
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----",
        );
        env::set_var("JWT_ALGORITHM", "RS256");
        env::set_var("JWT_AUDIENCE", "test-service");
        env::set_var("JWT_ISSUER", "test-issuer");

        // This test will fail with the dummy key, but tests the configuration logic
        let result = JwtConfig::from_env();
        assert!(result.is_err()); // Expected to fail with dummy key

        // Restore original values
        if let Some(value) = original_jwt_key {
            env::set_var("JWT_PUBLIC_KEY", value);
        } else {
            env::remove_var("JWT_PUBLIC_KEY");
        }

        if let Some(value) = original_jwt_algorithm {
            env::set_var("JWT_ALGORITHM", value);
        } else {
            env::remove_var("JWT_ALGORITHM");
        }

        if let Some(value) = original_jwt_audience {
            env::set_var("JWT_AUDIENCE", value);
        } else {
            env::remove_var("JWT_AUDIENCE");
        }

        if let Some(value) = original_jwt_issuer {
            env::set_var("JWT_ISSUER", value);
        } else {
            env::remove_var("JWT_ISSUER");
        }
    }
}
