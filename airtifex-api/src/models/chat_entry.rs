use crate::id::Uuid;
use crate::models::{Error, Result};
use crate::DbPool;
use airtifex_core::llm::ChatEntryType;

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum ChatEntryError {
    #[error("failed to create a chat entry - {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect a chat entry - {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete a chat entry - {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to list chat entries - {0}")]
    ListChatsError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChatEntry {
    #[serde(default)]
    pub entry_id: Uuid,
    pub chat_id: Uuid,
    pub entry_type: ChatEntryType,
    pub content: String,
    pub entry_date: chrono::DateTime<chrono::Utc>,
}

impl ChatEntry {
    pub fn new_user(chat_id: Uuid, content: String) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            chat_id,
            entry_type: ChatEntryType::User,
            content,
            entry_date: chrono::Utc::now(),
        }
    }
    pub fn new_bot(chat_id: Uuid, content: String) -> Self {
        Self {
            entry_id: Uuid::new_v4(),
            chat_id,
            entry_type: ChatEntryType::Bot,
            content,
            entry_date: chrono::Utc::now(),
        }
    }
}

impl ChatEntry {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chat_entries
                    (entry_id, chat_id, entry_type, content, entry_date)
            VALUES  ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&self.entry_id)
        .bind(&self.chat_id)
        .bind(self.entry_type)
        .bind(&self.content)
        .bind(&self.entry_date)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ChatEntryError::CreateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<()> {
        let mut tx = db.begin().await.map_err(ChatEntryError::DeleteError)?;
        sqlx::query(
            r#"
            DELETE FROM chat_entries
            WHERE entry_id = $1
            "#,
        )
        .bind(id)
        .execute(&mut tx)
        .await
        .map_err(ChatEntryError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| ())
            .map_err(ChatEntryError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn get_chat_entries(
        db: &DbPool,
        chat_id: &Uuid,
        username: &str,
    ) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            SELECT entry_id, chat_id, entry_type, content, entry_date
            FROM chat_entries
            INNER JOIN chats c ON c.id = $1
            WHERE chat_id = $1 AND c.username = $2
            ORDER BY entry_date
            "#,
        )
        .bind(chat_id)
        .bind(username)
        .fetch_all(db)
        .await
        .map_err(ChatEntryError::ListChatsError)
        .map_err(Error::from)
    }
}
