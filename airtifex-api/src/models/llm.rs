use crate::id::Uuid;
use crate::models::{Error, Result};
use crate::DbPool;

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum LlmError {
    #[error("failed to create a model- {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect a model- {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete a model- {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to list models - {0}")]
    ListLargeLanguageModelsError(sqlx::Error),
    #[error("failed to update a model - {0}")]
    UpdateError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct LargeLanguageModel {
    #[serde(default)]
    pub model_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl LargeLanguageModel {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            model_id: Uuid::new_v4(),
            name,
            description,
        }
    }

    pub fn id(&self) -> Uuid {
        self.model_id
    }
}

impl LargeLanguageModel {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO llm_models
                    (model_id, name, description)
            VALUES  ($1, $2, $3)
            "#,
        )
        .bind(&self.model_id)
        .bind(&self.name)
        .bind(&self.description)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(LlmError::CreateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<Self> {
        let mut tx = db.begin().await.map_err(LlmError::DeleteError)?;
        let deleted = sqlx::query_as(
            r#"
            DELETE FROM llm_models
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut tx)
        .await
        .map_err(LlmError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| deleted)
            .map_err(LlmError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn list(db: &DbPool) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
                    SELECT model_id, name, description
                    FROM llm_models
                    ORDER BY name
                "#,
        )
        .fetch_all(db)
        .await
        .map_err(LlmError::ListLargeLanguageModelsError)
        .map_err(Error::from)
    }

    pub async fn get_by_name(db: &DbPool, name: &str) -> Result<Self> {
        sqlx::query_as(
            r#"
                    SELECT model_id, name, description
                    FROM llm_models
                    WHERE name = $1
                "#,
        )
        .bind(name)
        .fetch_one(db)
        .await
        .map_err(LlmError::ListLargeLanguageModelsError)
        .map_err(Error::from)
    }
}
