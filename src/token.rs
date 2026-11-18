#![allow(dead_code)]
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_error::AppError;

/// Auxiliary function to obtain the encoding key from the environment.
pub fn get_encoding_key() -> anyhow::Result<EncodingKey> {
    let secret = std::env::var("JWT_PRIVATE_KEY")?;
    Ok(EncodingKey::from_rsa_pem(secret.as_bytes())?)
}

/// Auxiliary function to obtain the decoding key from the environment.
pub fn get_decoding_key() -> anyhow::Result<DecodingKey> {
    let public_key = std::env::var("JWT_PUBLIC_KEY")?;
    Ok(DecodingKey::from_rsa_pem(public_key.as_bytes())?)
}

/// JWT claims for the session.
#[derive(Serialize, Deserialize)]
struct SessionClaims {
    /// Subject, i.e. username.
    sub: String,
    /// The time at which the token should expire
    exp: u64,
}

/// Create a JWT for the application session.
pub fn produce_session_token(encoding_key: &EncodingKey, username: &str) -> anyhow::Result<String> {
    const SESSION_EXPIRY_SECS: u64 = 3600;

    let now = get_now_in_secs();
    let claims = SessionClaims {
        sub: username.to_string(),
        exp: now + SESSION_EXPIRY_SECS,
    };
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &claims,
        encoding_key,
    )?)
}

/// Verify the signature of the session JWT.
pub fn verify_session_token(
    decoding_key: &DecodingKey,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            AppError::Unauthorized("Missing or invalid authorization header".to_string())
        })?;

    decode::<SessionClaims>(token, decoding_key, &Validation::new(Algorithm::RS256))
        .map(|_| ())
        .map_err(|_| AppError::Unauthorized("Invalid session token".to_string()))
}

/// JWT claims for interacting with the Vonage APIs.
#[derive(Serialize)]
struct VonageClaims {
    /// Corresponds to the ID of the application with Network Registry capabilities, created in the user's dashboard.
    application_id: String,
    /// The time at which the token was issued
    iat: u64,
    /// The time at which the token should expire
    exp: u64,
    /// A unique identifier for the JWT
    jti: String,
}

/// Create a JWT for using the Vonage APIs.
pub fn produce_vonage_token() -> anyhow::Result<String> {
    const VONAGE_EXPIRY_SECS: u64 = 600;

    let secret = std::env::var("VONAGE_SECRET_KEY")?;
    let encoding_key = EncodingKey::from_rsa_pem(secret.as_bytes())?;
    let app_id = std::env::var("VONAGE_APP_ID")?;
    let now = get_now_in_secs();
    let vonage_claims = VonageClaims {
        iat: now,
        application_id: app_id,
        jti: Uuid::new_v4().to_string(),
        exp: now + VONAGE_EXPIRY_SECS,
    };
    Ok(encode(
        &Header::new(Algorithm::RS256),
        &vonage_claims,
        &encoding_key,
    )?)
}

fn get_now_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
