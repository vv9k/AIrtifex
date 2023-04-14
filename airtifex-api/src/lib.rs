#![feature(path_file_prefix)]
#![feature(let_chains)]
use crate::llm::{InferenceRequest, ModelName};
pub use airtifex_core::api_response::{ApiResponse, ApiVersion};
pub use errors::Error;

use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::Response;
use axum_extra::extract::cookie::Key;
use flume::Sender;
use std::ops::Deref;

#[macro_use]
mod guard;

pub mod auth;
pub mod config;
pub mod errors;
pub mod id;
pub mod llm;
pub mod models;
pub mod permissions;
pub mod routes;

#[cfg(feature = "postgres")]
pub type DbPool = sqlx::PgPool;
#[cfg(feature = "sqlite")]
pub type DbPool = sqlx::SqlitePool;

pub type Result<T> = core::result::Result<T, errors::Error>;

pub struct InnerAppState {
    pub uuid_context: id::V1Context,
    pub db: std::sync::Arc<crate::DbPool>,
    pub key: Key,
    pub config: config::Config,
    pub tx_inference_req: std::collections::HashMap<ModelName, Sender<InferenceRequest>>,
}

#[derive(Clone)]
pub struct SharedAppState(std::sync::Arc<InnerAppState>);

unsafe impl Sync for SharedAppState {}
unsafe impl Send for SharedAppState {}

impl Deref for SharedAppState {
    type Target = InnerAppState;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl FromRef<SharedAppState> for Key {
    fn from_ref(input: &SharedAppState) -> Self {
        input.key.clone()
    }
}

impl From<std::sync::Arc<InnerAppState>> for SharedAppState {
    fn from(value: std::sync::Arc<InnerAppState>) -> Self {
        Self(value)
    }
}

pub trait ToAxumResponse: Sized {
    fn into_response(self, code: StatusCode) -> Response;

    fn ok(self) -> Response {
        self.into_response(StatusCode::OK)
    }

    fn unauthorized(self) -> Response {
        self.into_response(StatusCode::UNAUTHORIZED)
    }

    fn bad_request(self) -> Response {
        self.into_response(StatusCode::BAD_REQUEST)
    }

    fn internal_server_error(self) -> Response {
        self.into_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl ToAxumResponse for ApiResponse {
    fn into_response(self, code: StatusCode) -> Response {
        use axum::response::IntoResponse;
        (code, axum::Json(self)).into_response()
    }
}
