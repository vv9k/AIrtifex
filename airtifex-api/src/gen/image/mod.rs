pub mod sd;

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{config::Config, models::image_model::ImageModel, DbPool, Result};

pub enum GenerateImageRequest {
    TextToImage(BaseImageData),
    ImageToImage(ImageToImageData),
    Inpaint(InpaintData),
}

impl GenerateImageRequest {
    pub fn id(&self) -> &str {
        match self {
            Self::TextToImage(data) => &data.id,
            Self::ImageToImage(data) => &data.data.id,
            Self::Inpaint(data) => &data.data.id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BaseImageData {
    pub id: String,
    pub prompt: String,
    pub width: i64,
    pub height: i64,
    pub n_steps: usize,
    pub seed: i64,
    pub num_samples: i64,
    pub guidance_scale: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageToImageData {
    pub data: BaseImageData,
    pub input_image: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InpaintData {
    pub data: BaseImageData,
    pub input_image: Vec<u8>,
    pub mask: Vec<u8>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SaveImageResult {
    pub id: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SaveImageFsResult {
    pub id: String,
    pub n_sample: i32,
    pub is_last: bool,
    pub path: std::path::PathBuf,
}

pub async fn initialize_models(
    db: Arc<DbPool>,
    config: &Config,
    runtime: Arc<Runtime>,
) -> Result<HashMap<String, flume::Sender<GenerateImageRequest>>> {
    tch::maybe_init_cuda();
    log::info!("Cuda available: {}", tch::Cuda::is_available());
    log::info!("Cudnn available: {}", tch::Cuda::cudnn_is_available());
    log::info!("MPS available: {}", tch::utils::has_mps());
    let mut txs = HashMap::new();
    for model_config in config.stable_diffusion.iter() {
        let model = model_config
            .name
            .clone()
            .unwrap_or_else(|| format!("stable-diffusion-{}", model_config.version.as_ref()));
        let exists = ImageModel::get_by_name(&db, &model).await.is_ok();

        log::info!("initializing image model {model}, exists in db: {exists}");

        if !exists {
            let image_model = ImageModel::new(
                model.clone(),
                model_config.model_description.clone(),
                model_config.features(),
            );
            image_model.create(&db).await?;
        }
        let tx_inference_req = sd::initialize(db.clone(), model_config.clone(), runtime.clone());
        txs.insert(model.clone(), tx_inference_req);
    }
    Ok(txs)
}
