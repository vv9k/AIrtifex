use crate::{
    id::Uuid,
    models::{Error, Result},
    DbPool,
};

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum ImageError {
    #[error("failed to create a image - {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect a image - {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete a image - {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to list images - {0}")]
    ListImagesError(sqlx::Error),
    #[error("failed to update a image - {0}")]
    UpdateError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Image {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model: String,
    pub width: i64,
    pub height: i64,
    pub prompt: String,
    pub input_image: Option<Vec<u8>>,
    pub mask: Option<Vec<u8>>,
    pub thumbnail: Option<Vec<u8>>,
    pub strength: Option<f64>,
    pub n_steps: i64,
    pub seed: i64,
    pub num_samples: i64,
    pub guidance_scale: f64,
    pub processing: bool,
    pub create_date: chrono::DateTime<chrono::Utc>,
}

impl Image {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: Uuid,
        model: String,
        width: i64,
        height: i64,
        prompt: String,
        input_image: Option<Vec<u8>>,
        mask: Option<Vec<u8>>,
        thumbnail: Option<Vec<u8>>,
        strength: Option<f64>,
        n_steps: i64,
        seed: i64,
        num_samples: i64,
        guidance_scale: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model,
            width,
            height,
            prompt,
            input_image,
            mask,
            thumbnail,
            strength,
            n_steps,
            seed,
            num_samples,
            guidance_scale,
            processing: true,
            create_date: chrono::Utc::now(),
        }
    }
}

impl Image {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO images
                    (id, user_id, model, width, height, prompt, input_image, mask, thumbnail, strength, n_steps, seed, num_samples, guidance_scale, processing, create_date)
            VALUES  ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(self.id)
        .bind(self.user_id)
        .bind(&self.model)
        .bind(self.width)
        .bind(self.height)
        .bind(&self.prompt)
        .bind(&self.input_image)
        .bind(&self.mask)
        .bind(&self.thumbnail)
        .bind(self.strength)
        .bind(self.n_steps)
        .bind(self.seed)
        .bind(self.num_samples)
        .bind(self.guidance_scale)
        .bind(self.processing)
        .bind(self.create_date)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ImageError::CreateError)
        .map_err(Error::from)
    }

    pub async fn list(db: &DbPool) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, model, width, height, prompt, input_image, mask, thumbnail, strength, n_steps, seed, num_samples, guidance_scale, processing, create_date
            FROM images
            "#,
        )
        .fetch_all(db)
        .await
        .map_err(ImageError::ListImagesError)
        .map_err(Error::from)
    }

    pub async fn get_by_id(db: &DbPool, id: &Uuid) -> Result<Self> {
        sqlx::query_as(
            r#"
            SELECT id, user_id, model, width, height, prompt, input_image, mask, thumbnail, strength, n_steps, seed, num_samples, guidance_scale, processing, create_date
            FROM images
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(ImageError::InspectError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<()> {
        let tx = db.begin().await.map_err(ImageError::DeleteError)?;
        sqlx::query(
            r#"
            DELETE FROM images
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(db)
        .await
        .map_err(ImageError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| ())
            .map_err(ImageError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn update_is_processing(db: &DbPool, id: &Uuid, processing: bool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE images
            SET processing = $1
            WHERE id = $2
            "#,
        )
        .bind(processing)
        .bind(id)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ImageError::UpdateError)
        .map_err(Error::from)
    }

    pub async fn update_thumbnail(db: &DbPool, id: &Uuid, thumbnail: &[u8]) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE images
            SET thumbnail = $1
            WHERE id = $2
            "#,
        )
        .bind(thumbnail)
        .bind(id)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ImageError::UpdateError)
        .map_err(Error::from)
    }
}
