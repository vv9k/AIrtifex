use crate::{
    id::Uuid,
    models::{Error, Result},
    DbPool,
};

use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

#[derive(Debug, ErrorType)]
pub enum ImageSampleError {
    #[error("failed to create a image sample - {0}")]
    CreateError(sqlx::Error),
    #[error("failed to inspect a image sample - {0}")]
    InspectError(sqlx::Error),
    #[error("failed to delete a image sample - {0}")]
    DeleteError(sqlx::Error),
    #[error("failed to get image sample - {0}")]
    GetImageError(sqlx::Error),
    #[error("failed to list image samples - {0}")]
    ListImagesError(sqlx::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ImageSample {
    #[serde(default)]
    pub sample_id: Uuid,
    pub image_id: Uuid,
    pub n: i32,
    pub data: Vec<u8>,
}

impl ImageSample {
    pub fn new(image_id: Uuid, n: i32, data: Vec<u8>) -> Self {
        Self {
            sample_id: Uuid::new_v4(),
            image_id,
            n,
            data,
        }
    }
}

impl ImageSample {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO image_samples
                    (sample_id, image_id, n, data)
            VALUES  ($1, $2, $3, $4)
            "#,
        )
        .bind(&self.sample_id)
        .bind(&self.image_id)
        .bind(&self.n)
        .bind(&self.data)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(ImageSampleError::CreateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<()> {
        let mut tx = db.begin().await.map_err(ImageSampleError::DeleteError)?;
        sqlx::query(
            r#"
            DELETE FROM image_samples
            WHERE sample_id = $1
            "#,
        )
        .bind(id)
        .execute(&mut tx)
        .await
        .map_err(ImageSampleError::DeleteError)?;

        tx.commit()
            .await
            .map(|_| ())
            .map_err(ImageSampleError::DeleteError)
            .map_err(Error::from)
    }

    pub async fn get_sample(db: &DbPool, image_id: &Uuid, n: i32) -> Result<Self> {
        sqlx::query_as(
            r#"
            SELECT sample_id, image_id, n, data
            FROM image_samples
            WHERE image_id = $1 AND n = $2
            "#,
        )
        .bind(image_id)
        .bind(n)
        .fetch_one(db)
        .await
        .map_err(ImageSampleError::GetImageError)
        .map_err(Error::from)
    }

    pub async fn get_image_samples(db: &DbPool, image_id: &Uuid) -> Result<Vec<Self>> {
        sqlx::query_as(
            r#"
            SELECT sample_id, image_id, n, data
            FROM image_samples
            INNER JOIN images i ON i.id = $1
            WHERE image_id = $1
            ORDER BY n
            "#,
        )
        .bind(image_id)
        .fetch_all(db)
        .await
        .map_err(ImageSampleError::ListImagesError)
        .map_err(Error::from)
    }
}
