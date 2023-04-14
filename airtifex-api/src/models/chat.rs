use crate::id::Uuid;
use crate::models::{chat_entry::ChatEntry, Error, Result};
use crate::DbPool;
use airtifex_core::llm::ChatSettings;

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum ChatError {
    #[error("failed to create a chat session - {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect a chat session - {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete a chat session - {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to list chat sessions - {0}")]
    ListChatsError(sqlx::Error),
    #[error("failed to update a chat - {0}")]
    UpdateError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chat {
    #[serde(default)]
    pub id: Uuid,
    pub username: String,
    pub title: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub model: String,
    pub num_predict: Option<i32>,
    pub system_prompt: Option<String>,
    pub n_batch: Option<i32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub temp: Option<f32>,
}

impl Chat {
    pub fn new(
        username: String,
        model: String,
        title: Option<String>,
        settings: ChatSettings,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            title: title.unwrap_or("New chat".into()),
            start_date: chrono::Utc::now(),
            model,
            num_predict: settings.num_predict.map(|n| n as i32),
            system_prompt: settings.system_prompt,
            n_batch: settings.n_batch.map(|n| n as i32),
            top_k: settings.top_k.map(|k| k as i32),
            top_p: settings.top_p,
            repeat_penalty: settings.repeat_penalty,
            temp: settings.temp,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn username(&self) -> &str {
        &self.username
    }
}

impl Chat {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chats
                    (id, username, title, start_date, model, num_predict, system_prompt, n_batch, top_k, top_p, repeat_penalty, temp)
            VALUES  ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&self.id)
        .bind(&self.username)
        .bind(&self.title)
        .bind(self.start_date)
        .bind(&self.model)
        .bind(&self.num_predict)
        .bind(&self.system_prompt)
        .bind(&self.n_batch)
        .bind(&self.top_k)
        .bind(&self.top_p)
        .bind(&self.repeat_penalty)
        .bind(&self.temp)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ChatError::CreateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<()> {
        let tx = db.begin().await.map_err(ChatError::DeleteError)?;
        sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(db)
        .await
        .map_err(ChatError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| ())
            .map_err(ChatError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn get_chat_for_user(db: &DbPool, username: &str, chat_id: &Uuid) -> Result<Self> {
        sqlx::query_as(
            r#"
                    SELECT id, username, title, start_date, model, num_predict, system_prompt, n_batch, top_k, top_p, repeat_penalty, temp
                    FROM chats
                    WHERE id = $1 AND username = $2
                "#,
        )
        .bind(chat_id)
        .bind(username)
        .fetch_one(db)
        .await
        .map_err(ChatError::InspectError)
        .map_err(Error::from)
    }

    pub async fn list_chats_of_user(db: &DbPool, username: &str) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
                    SELECT id, username, title, start_date, model, num_predict, system_prompt, n_batch, top_k, top_p, repeat_penalty, temp
                    FROM chats
                    WHERE username = $1
                    ORDER BY start_date
                "#,
        )
        .bind(username)
        .fetch_all(db)
        .await
        .map_err(ChatError::ListChatsError)
        .map_err(Error::from)
    }

    pub async fn list_entries(db: &DbPool, id: &Uuid, username: &str) -> Result<Vec<ChatEntry>> {
        ChatEntry::get_chat_entries(db, id, username).await
    }

    pub async fn update_title(db: &DbPool, id: &Uuid, title: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE chats
            SET title = $1
            WHERE id = $2
            "#,
        )
        .bind(title)
        .bind(id)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ChatError::UpdateError)
        .map_err(Error::from)
    }
}
