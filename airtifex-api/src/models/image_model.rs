use crate::id::Uuid;
use crate::models::{Error, Result};
use crate::DbPool;

use airtifex_core::image::ImageModelFeatures;
use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum ImageModelError {
    #[error("failed to create image model- {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect image model- {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete image model- {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to list image models - {0}")]
    ListImageModelsError(sqlx::Error),
    #[error("failed to update image model - {0}")]
    UpdateError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImageModel {
    #[serde(default)]
    pub model_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub feature_inpaint: bool,
    pub feature_text_to_image: bool,
    pub feature_image_to_image: bool,
}

impl ImageModel {
    pub fn new(name: String, description: Option<String>, features: ImageModelFeatures) -> Self {
        Self {
            model_id: Uuid::new_v4(),
            name,
            description,
            feature_inpaint: features.inpaint,
            feature_text_to_image: features.text_to_image,
            feature_image_to_image: features.image_to_image,
        }
    }

    pub fn id(&self) -> Uuid {
        self.model_id
    }
}

impl ImageModel {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO image_models
                    (model_id, name, description, feature_inpaint, feature_text_to_image, feature_image_to_image)
            VALUES  ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&self.model_id)
        .bind(&self.name)
        .bind(&self.description)
        .bind(self.feature_inpaint)
        .bind(self.feature_text_to_image)
        .bind(self.feature_image_to_image)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ImageModelError::CreateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<Self> {
        let mut tx = db.begin().await.map_err(ImageModelError::DeleteError)?;
        let deleted = sqlx::query_as(
            r#"
            DELETE FROM image_models
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut tx)
        .await
        .map_err(ImageModelError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| deleted)
            .map_err(ImageModelError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn list(db: &DbPool) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
                    SELECT model_id, name, description, feature_inpaint, feature_text_to_image, feature_image_to_image
                    FROM image_models
                    ORDER BY name
                "#,
        )
        .fetch_all(db)
        .await
        .map_err(ImageModelError::ListImageModelsError)
        .map_err(Error::from)
    }

    pub async fn get_by_name(db: &DbPool, name: &str) -> Result<Self> {
        sqlx::query_as(
            r#"
                    SELECT model_id, name, description, feature_inpaint, feature_text_to_image, feature_image_to_image
                    FROM image_models
                    WHERE name = $1
                "#,
        )
        .bind(name)
        .fetch_one(db)
        .await
        .map_err(ImageModelError::ListImageModelsError)
        .map_err(Error::from)
    }
}
