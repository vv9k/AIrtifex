use serde::{Deserialize, Serialize};

pub mod api_response;
pub mod auth;
pub mod llm;
pub mod query;
pub mod user;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonWebToken {
    pub token: String,
}
