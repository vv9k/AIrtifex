use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TextToImageRequest {
    pub prompt: String,
    pub model: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub n_steps: Option<usize>,
    pub seed: Option<i64>,
    pub num_samples: Option<i64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TextToImageResponse {
    pub image_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ImageInspect {
    pub id: String,
    pub user_id: String,
    pub model: String,
    pub width: i64,
    pub height: i64,
    pub prompt: String,
    pub n_steps: i64,
    pub seed: i64,
    pub num_samples: i64,
    pub processing: bool,
    pub create_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ImageSampleInspect {
    pub sample_id: String,
    pub image_id: String,
    pub n_sample: i32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageModelListEntry {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
}
