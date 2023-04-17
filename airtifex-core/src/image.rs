use debug_stub_derive::DebugStub;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize, DebugStub)]
pub struct ImageGenerateRequest {
    pub prompt: String,
    #[debug_stub = "InputImage"]
    pub input_image: Option<Vec<u8>>,
    #[debug_stub = "Mask"]
    pub mask: Option<Vec<u8>>,
    pub model: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub n_steps: Option<usize>,
    pub seed: Option<i64>,
    pub num_samples: Option<i64>,
    pub guidance_scale: Option<f64>,
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
    pub input_image: Option<Vec<u8>>,
    pub mask: Option<Vec<u8>>,
    pub n_steps: i64,
    pub seed: i64,
    pub num_samples: i64,
    pub guidance_scale: f64,
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
    pub features: ImageModelFeatures,
}

fn on() -> bool {
    true
}

fn off() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageModelFeatures {
    #[serde(default = "off")]
    pub inpaint: bool,
    #[serde(default = "on")]
    pub text_to_image: bool,
    #[serde(default = "on")]
    pub image_to_image: bool,
}
