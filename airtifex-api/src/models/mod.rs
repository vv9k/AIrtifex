pub mod chat;
pub mod chat_entry;
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
    LlmError(#[from] llm::LlmError),
    #[error(transparent)]
    ChatEntryError(#[from] chat_entry::ChatEntryError),
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
