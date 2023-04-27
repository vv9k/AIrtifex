use crate::{errors::Error, ApiResponse, SharedAppState};
use airtifex_core::user::AccountType;

use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::extract::cookie::{Key, PrivateCookieJar};
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, errors::Error as JwtError, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

const KEY_VALID_DURATION: i64 = 3600;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let keys = Keys::new(secret.as_bytes());
    std::env::remove_var("JWT_SECRET");
    keys
});

#[derive(Debug, ErrorType)]
pub enum AuthError {
    #[error("Invalid token - {0}")]
    InvalidToken(JwtError),
    #[error("Invalid path")]
    InvalidPath,
    #[error("Invalid authorization header")]
    InvalidHeader,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(ApiResponse::failure(self))).into_response()
    }
}

#[derive(Debug, ErrorType)]
pub enum TokenGenerationError {
    #[error("Failed to generate valid timestamp for a authentication token")]
    TimestampGenerationFailed,
    #[error("Failed to encode auth token - {0}")]
    EncodingFailed(#[from] JwtError),
}

#[derive(Debug, ErrorType)]
pub enum InvalidTokenError {
    #[error("Failed to decode auth token - {0}")]
    DecodingFailed(#[from] JwtError),
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[async_trait]
impl FromRequestParts<SharedAppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedAppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = PrivateCookieJar::<Key>::from_request_parts(parts, state)
            .await
            .ok();

        let token = if let Some(cookie_token) = cookies.and_then(|c| c.get("Bearer")) {
            log::trace!("got auth token from cookie");
            cookie_token.value().to_owned()
        } else {
            let TypedHeader(Authorization(bearer)) =
                match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
                    Ok(bearer) => bearer,
                    Err(e) => {
                        log::error!("Failed to extract Authorization Bearer header - {e}");
                        return Err(AuthError::InvalidHeader);
                    }
                };
            log::trace!("got auth token from header");
            bearer.token().to_owned()
        };
        let token_data =
            match decode::<Claims>(&token, &KEYS.decoding, &Validation::new(Algorithm::HS512)) {
                Ok(bearer) => bearer,
                Err(e) => {
                    log::error!("Failed to decode claims from token - {e}");
                    return Err(AuthError::InvalidToken(e));
                }
            };

        Ok(token_data.claims)
    }
}

pub trait Token {
    fn token(&self) -> &str;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonWebToken {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn generate_jwt(user: &str, role: AccountType) -> Result<String, Error> {
    let exp = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(KEY_VALID_DURATION))
        .ok_or(TokenGenerationError::TimestampGenerationFailed)?
        .timestamp();

    let claims = Claims {
        sub: user.to_string(),
        role: role.as_ref().to_string(),
        exp: exp as usize,
    };

    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &KEYS.encoding)
        .map_err(TokenGenerationError::from)
        .map_err(Error::from)
}
