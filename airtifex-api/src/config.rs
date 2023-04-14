use crate::{Error, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Deserialize, Serialize)]
struct RawConfig {
    listen_addr: String,
    listen_port: u16,
    db_url: String,
    jwt_secret: String,
    llms: Vec<LlmConfig>,
}

fn default_num_ctx_tokens() -> usize {
    1024
}
fn default_batch_size() -> usize {
    8
}
fn default_repeat_last_n() -> usize {
    64
}
fn default_repeat_penalty() -> f32 {
    1.30
}
fn default_temperature() -> f32 {
    0.80
}
fn default_top_k() -> usize {
    40
}
fn default_top_p() -> f32 {
    0.95
}
fn default_max_inference_sessions() -> usize {
    5
}
fn default_num_threads() -> usize {
    num_cpus::get_physical()
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LlmConfig {
    pub model_description: Option<String>,
    pub model_path: std::path::PathBuf,
    #[serde(default = "default_num_ctx_tokens")]
    /// Sets the size of the context (in tokens). Allows feeding longer prompts.
    /// Note that this affects memory.
    pub num_ctx_tokens: usize,
    #[serde(default = "default_num_threads")]
    pub num_threads: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_repeat_last_n")]
    pub repeat_last_n: usize,
    #[serde(default = "default_repeat_penalty")]
    pub repeat_penalty: f32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub float16: bool,
    pub seed: Option<u64>,
    #[serde(default = "default_max_inference_sessions")]
    // Maximum concurent sessions for inference
    pub max_inference_sessions: usize,
}

pub struct Config {
    pub listen_addr: std::net::IpAddr,
    pub listen_port: u16,
    pub db_url: String,
    pub jwt_secret: String,
    pub llms: HashMap<String, LlmConfig>,
}

impl Config {
    pub fn read(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let data = std::fs::read(path).map_err(Error::ConfigReadFailed)?;

        let config: RawConfig =
            serde_yaml::from_slice(&data).map_err(Error::ConfigDeserializeFailed)?;

        let addr = env::var("AIRTIFEX_LISTEN_ADDR")
            .ok()
            .unwrap_or(config.listen_addr);
        let listen_addr = addr
            .parse::<std::net::IpAddr>()
            .map_err(|e| Error::InvalidIp(e.to_string()))?;

        let listen_port = env::var("AIRTIFEX_LISTEN_PORT")
            .ok()
            .and_then(|port| port.parse().ok())
            .unwrap_or(config.listen_port);

        let db_url = env::var("AIRTIFEX_DB_URL").ok().unwrap_or(config.db_url);

        let jwt_secret = env::var("AIRTIFEX_JWT_SECRET")
            .ok()
            .unwrap_or(config.jwt_secret);

        let llms = config
            .llms
            .into_iter()
            .enumerate()
            .map(|(i, cfg)| {
                let name = cfg
                    .model_path
                    .file_prefix()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("llm-model-{i}"));
                (name, cfg)
            })
            .collect();

        Ok(Self {
            listen_addr,
            listen_port,
            db_url,
            jwt_secret,
            llms,
        })
    }
}
