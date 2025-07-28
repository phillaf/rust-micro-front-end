#![allow(clippy::uninlined_format_args)]

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, header::COOKIE, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        let mut public_key_pem =
            env::var("JWT_PUBLIC_KEY").map_err(|_| "JWT_PUBLIC_KEY environment variable not set")?;

        // Remove surrounding single or double quotes if present
        if (public_key_pem.starts_with('"') && public_key_pem.ends_with('"'))
            || (public_key_pem.starts_with('\'') && public_key_pem.ends_with('\''))
        {
            public_key_pem = public_key_pem[1..public_key_pem.len() - 1].to_string();
        }

        // Handle escaped newlines in the public key
        if public_key_pem.contains("\\n") {
            public_key_pem = public_key_pem.replace("\\n", "\n");
        }

        tracing::debug!("Processing public key PEM ({} chars): {}", public_key_pem.len(), public_key_pem);

        let algorithm = env::var("JWT_ALGORITHM").unwrap_or_else(|_| "RS256".to_string());
        let audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "micro-frontend-service".to_string());
        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "test-auth-service".to_string());

        let public_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| format!("Failed to parse JWT public key: {e}"))?;

        tracing::debug!("Successfully parsed RSA public key");

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
    tracing::info!("JWT authentication middleware started");

    // Try to extract JWT token from multiple sources
    let token = extract_jwt_token(&request).ok_or_else(|| {
        tracing::info!("No JWT token found in request");
        StatusCode::UNAUTHORIZED
    })?;

    tracing::info!("JWT token found, length: {}, starting validation", token.len());

    // Validate JWT token
    let jwt_config = JwtConfig::from_env().map_err(|e| {
        tracing::error!("JWT config error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        "JWT config loaded, algorithm: {:?}, leeway: {}",
        jwt_config.validation.algorithms,
        jwt_config.validation.leeway
    );

    match decode::<Claims>(&token, &jwt_config.public_key, &jwt_config.validation) {
        Ok(token_data) => {
            tracing::info!(
                "JWT token validated successfully. Subject: {}, Expiry: {}, Issuer: {:?}",
                token_data.claims.sub,
                token_data.claims.exp,
                token_data.claims.iss
            );
            request.extensions_mut().insert(token_data.claims);
            Ok(next.run(request).await)
        }
        Err(e) => {
            tracing::error!("JWT validation failed: {}", e);

            // Log extra information for debugging JWT issues
            match &e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::error!("JWT token has expired");
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::error!("JWT token has invalid signature - check public key");
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    tracing::error!("JWT token is invalid");
                }
                other => {
                    tracing::error!("JWT error kind: {:?}", other);
                }
            }

            // We return a basic 401 to avoid leaking too much info to clients
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

fn extract_jwt_token(request: &Request) -> Option<String> {
    tracing::debug!("Extracting JWT token from request");

    // Define a function to validate token expiry before using it
    // This is to prevent using expired tokens when multiple tokens are present
    let is_valid_token = |token: &str| -> bool {
        // Basic validation: check if it has three parts with dots
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        // Try to decode payload part without signature validation
        if let Ok(payload) = URL_SAFE_NO_PAD.decode(parts[1]) {
            if let Ok(payload_str) = String::from_utf8(payload) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                    // Check if token is expired
                    if let Some(exp) = json.get("exp").and_then(|e| e.as_i64()) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        return exp > now;
                    }
                }
            }
        }

        // If we can't validate expiry, proceed with it anyway
        true
    };

    // Collect all tokens from different sources
    let mut potential_tokens = Vec::new();

    // 1. Check for jwt_token cookie
    if let Some(cookie_header) = request.headers().get(COOKIE).and_then(|header| header.to_str().ok()) {
        tracing::info!("Found cookies header: {}", cookie_header);

        for cookie_part in cookie_header.split(';') {
            let cookie_part = cookie_part.trim();
            tracing::debug!("Processing cookie part: {}", cookie_part);

            if let Some(token) = cookie_part.strip_prefix("jwt_token=") {
                tracing::info!("Found JWT token in cookie, length: {}", token.len());
                potential_tokens.push(token.to_string());
            }
        }
    }

    // 2. Check Authorization header (Bearer token)
    if let Some(auth_header) = request.headers().get(AUTHORIZATION).and_then(|header| header.to_str().ok()) {
        tracing::debug!("Found Authorization header: {}", auth_header);
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            tracing::info!("Found JWT token in Authorization header, length: {}", token.len());
            potential_tokens.push(token.to_string());
        }
    }

    // Return the first valid token, prioritizing non-expired tokens
    for token in &potential_tokens {
        if is_valid_token(token) {
            tracing::info!("Using valid, non-expired token of length {}", token.len());
            return Some(token.clone());
        }
    }

    // If no valid tokens found, return the first token as a fallback
    if !potential_tokens.is_empty() {
        tracing::warn!("No valid tokens found, using first token as fallback");
        return Some(potential_tokens[0].clone());
    }

    tracing::debug!("No JWT token found in request");
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
