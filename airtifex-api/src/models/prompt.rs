use crate::{
    id::Uuid,
    models::{Error, Result},
    DbPool,
};
use airtifex_core::llm::InferenceSettings;

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum PromptError {
    #[error("failed to create a prompt - {0}")]
    Create(sqlx::Error),
    #[error("failed to inspect a prompt - {0}")]
    Inspect(sqlx::Error),
    #[error("failed to delete a prompt - {0}")]
    Delete(sqlx::Error),
    #[error("failed to list prompts - {0}")]
    List(sqlx::Error),
    #[error("failed to update a prompt - {0}")]
    Update(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Prompt {
    #[serde(default)]
    pub id: Uuid,
    pub username: String,
    pub prompt: String,
    pub response: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub model: String,
    pub num_predict: Option<i32>,
    pub n_batch: Option<i32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub temp: Option<f32>,
}

impl Prompt {
    pub fn new(
        username: String,
        model: String,
        prompt: String,
        response: String,
        settings: InferenceSettings,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            prompt,
            response,
            date: chrono::Utc::now(),
            model,
            num_predict: settings.num_predict.map(|n| n as i32),
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
}

impl Prompt {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO prompts
                    (id, username, prompt, response, date, model, num_predict, n_batch, top_k, top_p, repeat_penalty, temp)
            VALUES  ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
        )
        .bind(&self.id)
        .bind(&self.username)
        .bind(&self.prompt)
        .bind(&self.response)
        .bind(self.date)
        .bind(&self.model)
        .bind(&self.num_predict)
        .bind(&self.n_batch)
        .bind(&self.top_k)
        .bind(&self.top_p)
        .bind(&self.repeat_penalty)
        .bind(&self.temp)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(PromptError::Create)
        .map_err(Error::from)
    }

    pub async fn delete_prompt_for_user(db: &DbPool, username: &str, id: &Uuid) -> Result<()> {
        let tx = db.begin().await.map_err(PromptError::Delete)?;
        sqlx::query(
            r#"
            DELETE FROM prompts
            WHERE id = $1 AND username = $2
            "#,
        )
        .bind(id)
        .bind(username)
        .execute(db)
        .await
        .map_err(PromptError::Delete)?;

        tx.commit()
            .await
            .map(|_| ())
            .map_err(PromptError::Delete)
            .map_err(Error::from)
    }

    pub async fn get_prompt_for_user(db: &DbPool, username: &str, chat_id: &Uuid) -> Result<Self> {
        sqlx::query_as(
            r#"
                    SELECT id, username, prompt, response, date, model, num_predict, n_batch, top_k, top_p, repeat_penalty, temp
                    FROM prompts
                    WHERE id = $1 AND username = $2
                "#,
        )
        .bind(chat_id)
        .bind(username)
        .fetch_one(db)
        .await
        .map_err(PromptError::Inspect)
        .map_err(Error::from)
    }

    pub async fn list_prompts_of_user(db: &DbPool, username: &str) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
                    SELECT id, username, prompt, response, date, model, num_predict, n_batch, top_k, top_p, repeat_penalty, temp
                    FROM prompts
                    WHERE username = $1
                    ORDER BY date
                "#,
        )
        .bind(username)
        .fetch_all(db)
        .await
        .map_err(PromptError::List)
        .map_err(Error::from)
    }
}
