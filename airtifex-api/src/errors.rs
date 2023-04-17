use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Failed to initialize the server: {0}")]
    ServerInitializationError(std::io::Error),
    #[error("Error: {0}")]
    ServerError(std::io::Error),
    #[error("Failed to read environment variable `{0}`: {1}")]
    EnvVarError(String, std::env::VarError),
    #[error("The provided port is not a valid number - {0}")]
    InvalidPort(String),
    #[error("The provided IP is not valid - {0}")]
    InvalidIp(String),
    #[error(transparent)]
    CredentialsError(#[from] crate::auth::AuthError),
    #[error(transparent)]
    AuthenticationError(#[from] crate::models::user::AuthenticationError),
    #[error(transparent)]
    TokenGenerationError(#[from] crate::auth::TokenGenerationError),
    #[error(transparent)]
    InvalidTokenError(#[from] crate::auth::InvalidTokenError),
    #[error(transparent)]
    ModelError(#[from] crate::models::Error),
    #[error(transparent)]
    TextToImageError(#[from] crate::gen::image::sd::GenImageError),
    #[error(transparent)]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    HyperError(#[from] hyper::Error),
    #[error("failed to read configuration file - {0}")]
    ConfigReadFailed(std::io::Error),
    #[error("failed to deserialize configuration file as yaml - {0}")]
    ConfigDeserializeFailed(serde_yaml::Error),
    #[error("Failed to send token to receiver - {0}")]
    InferenceSend(flume::SendError<airtifex_core::llm::ChatStreamResult>),
    #[error(transparent)]
    InferenceError(#[from] llama_rs::InferenceError),
}
