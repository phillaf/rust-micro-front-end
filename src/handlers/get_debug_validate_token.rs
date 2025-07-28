use anyhow::Result;
use axum::{extract::Path, http::StatusCode, response::Html};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::env;

use crate::middleware::jwt_auth::Claims;

pub async fn get_debug_validate_token(Path(token): Path<String>) -> Result<Html<String>, StatusCode> {
    // Read JWT configuration from environment
    let mut public_key_pem = env::var("JWT_PUBLIC_KEY").map_err(|_| {
        tracing::error!("JWT_PUBLIC_KEY environment variable not set");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Remove surrounding single or double quotes if present
    if (public_key_pem.starts_with('"') && public_key_pem.ends_with('"'))
        || (public_key_pem.starts_with('\'') && public_key_pem.ends_with('\''))
    {
        public_key_pem = public_key_pem[1..public_key_pem.len() - 1].to_string();
    }

    // Handle escaped newlines in the public key (for .env with \n)
    if public_key_pem.contains("\\n") {
        public_key_pem = public_key_pem.replace("\\n", "\n");
    }

    tracing::debug!("Public key PEM loaded ({} chars): {}", public_key_pem.len(), public_key_pem);

    let algorithm = env::var("JWT_ALGORITHM").unwrap_or_else(|_| "RS256".to_string());
    let audience = env::var("JWT_AUDIENCE").unwrap_or_else(|_| "micro-frontend-service".to_string());
    let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "test-auth-service".to_string());

    // Set up JWT validation
    let public_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).map_err(|e| {
        tracing::error!("Failed to parse JWT public key: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut validation = Validation::new(match algorithm.as_str() {
        "RS256" => Algorithm::RS256,
        "ES256" => Algorithm::ES256,
        _ => {
            tracing::error!("Unsupported JWT algorithm: {}", algorithm);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    });

    validation.set_audience(&[audience.clone()]);
    validation.set_issuer(&[issuer.clone()]);
    validation.leeway = 60; // 1 minute clock skew

    tracing::debug!("Token validation configuration: {:?}", validation);

    // Try to validate the token
    let validation_result = decode::<Claims>(&token, &public_key, &validation);

    tracing::debug!("Validation result: {:?}", validation_result);

    // Create HTML response with validation details
    let html = match validation_result {
        Ok(token_data) => {
            let claims = token_data.claims;
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>JWT Token Validation</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        .success {{ background-color: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 15px; border-radius: 4px; }}
        .token-info {{ background-color: #f8f9fa; padding: 15px; border-radius: 4px; margin-top: 20px; }}
        .token-info pre {{ white-space: pre-wrap; word-break: break-all; }}
    </style>
</head>
<body>
    <h1>JWT Token Validation</h1>
    <div class="success">✅ Token is valid!</div>
    <div class="token-info">
        <h2>Token Information</h2>
        <ul>
            <li><strong>Subject (Username):</strong> {subject}</li>
            <li><strong>Issued at:</strong> {issued_at}</li>
            <li><strong>Expires at:</strong> {expires_at}</li>
            <li><strong>Audience:</strong> {audience}</li>
            <li><strong>Issuer:</strong> {issuer}</li>
        </ul>
        <h2>Raw Token</h2>
        <pre>{token}</pre>
        <h2>Debug Info</h2>
        <pre>Token header type: {header_type}
Token algorithm: {header_alg}</pre>
    </div>
</body>
</html>
                "#,
                subject = claims.sub,
                issued_at = claims.iat,
                expires_at = claims.exp,
                audience = claims.aud,
                issuer = claims.iss,
                token = token,
                header_type = token_data.header.typ.unwrap_or_default(),
                header_alg = format!("{:?}", token_data.header.alg)
            )
        }
        Err(err) => {
            format!(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>JWT Token Validation</title>
    <style>
        body {{ font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        .error {{ background-color: #f8d7da; border: 1px solid #f5c6cb; color: #721c24; padding: 15px; border-radius: 4px; }}
        .config {{ background-color: #f8f9fa; padding: 15px; border-radius: 4px; margin-top: 20px; }}
        .token-info pre {{ white-space: pre-wrap; word-break: break-all; }}
        .details {{ font-family: monospace; margin-top: 10px; padding: 10px; background: #f8f9fa; }}
    </style>
</head>
<body>
    <h1>JWT Token Validation</h1>
    <div class="error">❌ Token validation failed: {error}</div>
    <div class="config">
        <h2>Validation Configuration</h2>
        <ul>
            <li><strong>Algorithm:</strong> {algorithm}</li>
            <li><strong>Expected Audience:</strong> {audience}</li>
            <li><strong>Expected Issuer:</strong> {issuer}</li>
        </ul>
        <h2>Token Details</h2>
        <div class="details">
            <p>Token Parts:</p>
            <ul>
                <li>Header: {header}</li>
                <li>Payload: {payload}</li>
                <li>Signature: {signature}</li>
            </ul>
        </div>
        <h2>Raw Token</h2>
        <pre>{token}</pre>
    </div>
</body>
</html>
                "#,
                error = err,
                algorithm = algorithm,
                audience = audience,
                issuer = issuer,
                token = token,
                header = token.split('.').nth(0).unwrap_or("invalid"),
                payload = token.split('.').nth(1).unwrap_or("invalid"),
                signature = token.split('.').nth(2).unwrap_or("invalid")
            )
        }
    };

    Ok(Html(html))
}
