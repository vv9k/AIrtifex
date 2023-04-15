pub mod chat;
pub mod chat_entry;
pub mod image;
pub mod image_model;
pub mod image_sample;
pub mod llm;
pub mod user;

use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    AuthenticationError(#[from] user::AuthenticationError),
    #[error(transparent)]
    UserError(#[from] user::UserError),
    #[error(transparent)]
    ChatError(#[from] chat::ChatError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    ImageModelError(#[from] image_model::ImageModelError),
    #[error(transparent)]
    LlmError(#[from] llm::LlmError),
    #[error(transparent)]
    ChatEntryError(#[from] chat_entry::ChatEntryError),
    #[error(transparent)]
    ImageSampleError(#[from] image_sample::ImageSampleError),
}

pub async fn run_pragma(db: &crate::DbPool) -> crate::Result<()> {
    sqlx::query(
        r#"
            PRAGMA foreign_keys = ON;
            "#,
    )
    .execute(db)
    .await
    .map(|_| ())
    .map_err(crate::Error::from)
}
