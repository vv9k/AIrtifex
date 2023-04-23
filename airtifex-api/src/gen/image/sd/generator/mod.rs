pub mod img2img;
pub mod inpaint;
pub mod txt2img;

use crate::config::{StableDiffusionConfig, StableDiffusionVersion};
use crate::gen::image::{BaseImageData, SaveImageFsResult};
use crate::Result;

use diffusers::models::unet_2d::UNet2DConditionModel;
use diffusers::models::vae::AutoEncoderKL;
use diffusers::pipelines::stable_diffusion;
use diffusers::schedulers::ddim::DDIMScheduler;
use diffusers::transformers::clip;
use flume::Sender;
use log::Level;
use std::path::{Path, PathBuf};
use tch::Kind;
use tch::{nn::Module, Device, Tensor};

pub const LATENTS_SCALE: f64 = 0.18215;

#[derive(Debug, thiserror::Error)]
pub enum GenImageError {
    #[error("failed to create CLIP Tokenizer - {0}")]
    ClipTokenizerInit(anyhow::Error),
    #[error("failed to encode prompt - {0}")]
    PromptEncode(anyhow::Error),
    #[error("failed to encode tokens - {0}")]
    TokensEncode(anyhow::Error),
    #[error("failed to build CLIP Transformer - {0}")]
    ClipTransformerBuild(anyhow::Error),
    #[error("failed to build autoencoder - {0}")]
    VaeBuild(anyhow::Error),
    #[error("failed to build unet - {0}")]
    UnetBuild(anyhow::Error),
    #[error("failed to load input image - {0}")]
    LoadInputImage(tch::TchError),
    #[error("failed to load mask - {0}")]
    LoadMask(tch::TchError),
    #[error("failed to 3D size for a tensor - {0}")]
    Get3dSize(tch::TchError),
    #[error("failed to resize image- {0}")]
    ImageResize(tch::TchError),
}

pub trait ImageGenerator {
    fn type_(&self) -> &'static str;
    fn base_generator(&self) -> &BaseImageGenerator;
    fn is_finished(&self) -> bool;
    fn process_next_timestep(&mut self) -> bool;

    fn log_timestep(&self) {
        self.base_generator().log_timestep(self.type_());
    }

    fn log(&self, level: Level, msg: &str) {
        self.base_generator().log(level, self.type_(), msg);
    }
}

pub struct BaseImageGenerator {
    vae: AutoEncoderKL,
    vae_device: Device,
    unet: UNet2DConditionModel,
    unet_device: Device,
    scheduler: DDIMScheduler,
    text_embeddings: Tensor,
    tx_results: Sender<SaveImageFsResult>,
    request: BaseImageData,
    sd_config: stable_diffusion::StableDiffusionConfig,
    save_dir: PathBuf,
    processed_samples: usize,
    processed_timesteps: usize,
    bsize: i64,
}

impl BaseImageGenerator {
    pub fn new(
        request: BaseImageData,
        config: &StableDiffusionConfig,
        clip_device: Device,
        unet_device: Device,
        vae_device: Device,
        tx_results: Sender<SaveImageFsResult>,
        save_dir: impl AsRef<Path>,
        bsize: i64,
    ) -> Result<Self> {
        let sd_config = match config.version {
            StableDiffusionVersion::V2_1 => stable_diffusion::StableDiffusionConfig::v2_1(
                None,
                Some(request.height),
                Some(request.width),
            ),
            _ => stable_diffusion::StableDiffusionConfig::v1_5(
                None,
                Some(request.height),
                Some(request.width),
            ),
        };
        let scheduler = sd_config.build_scheduler(request.n_steps);
        let tokenizer = clip::Tokenizer::create(&config.vocab_file, &sd_config.clip)
            .map_err(GenImageError::ClipTokenizerInit)?;

        log::debug!(
            "[{}] Generating image for prompt \"{}\".",
            request.id,
            request.prompt
        );
        let tokens = tokenizer
            .encode(&request.prompt)
            .map_err(GenImageError::PromptEncode)?;

        let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
        let tokens = Tensor::of_slice(&tokens).view((1, -1)).to(clip_device);
        let uncond_tokens = tokenizer.encode("").map_err(GenImageError::TokensEncode)?;
        let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
        let uncond_tokens = Tensor::of_slice(&uncond_tokens)
            .view((1, -1))
            .to(clip_device);

        let no_grad_guard = tch::no_grad_guard();

        log::debug!("[{}] Building the Clip transformer.", request.id);
        let text_model = sd_config
            .build_clip_transformer(&config.clip_weights_path.to_string_lossy(), clip_device)
            .map_err(GenImageError::ClipTransformerBuild)?;

        let text_embeddings = text_model.forward(&tokens);
        let uncond_embeddings = text_model.forward(&uncond_tokens);
        let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(unet_device);

        log::debug!("[{}] Building the autoencoder", request.id);
        let vae = sd_config
            .build_vae(&config.vae_weights_path.to_string_lossy(), vae_device)
            .map_err(GenImageError::VaeBuild)?;

        log::debug!("[{}] Building unet", request.id);
        let unet = sd_config
            .build_unet(&config.unet_weights_path.to_string_lossy(), unet_device, 4)
            .map_err(GenImageError::UnetBuild)?;

        drop(no_grad_guard);

        Ok(Self {
            vae,
            vae_device,
            unet,
            unet_device,
            scheduler,
            text_embeddings,
            tx_results,
            request,
            sd_config,
            save_dir: save_dir.as_ref().to_path_buf(),
            processed_samples: 0,
            processed_timesteps: 0,
            bsize,
        })
    }

    pub fn sample_idx(&self) -> i64 {
        self.processed_samples as i64
    }

    pub fn is_finished(&self) -> bool {
        self.processed_samples as i64 >= self.request.num_samples
    }

    pub fn decode_latents(&self, latents: &Tensor) -> Tensor {
        let decoded = self.vae.decode(&(latents / LATENTS_SCALE));
        let decoded = (decoded / 2 + 0.5).clamp(0., 1.).to_device(Device::Cpu);
        (decoded * 255.).to_kind(Kind::Uint8)
    }

    pub fn save_image(&mut self, image: Tensor) {
        let idx = self.sample_idx() + 1;
        let path = self.save_dir.join(format!("{}-{idx}.png", self.request.id));
        let thumbnail_path = self
            .save_dir
            .join(format!("{}-{idx}-tbn.png", self.request.id));
        log::info!("saving image");
        if let Err(e) = tch::vision::image::save(&image, &path) {
            log::error!(
                "[{}][{}/{}] failed to save image to file - {e}",
                self.request.id,
                idx,
                self.request.num_samples
            );
        }
        log::info!("generating image thumbnail");
        let image_data = tch::vision::image::load(&path).unwrap();
        match generate_thumbnail(&image_data, 64, 64) {
            Ok(thumbnail) => {
                log::info!("saving image thumbnail");
                if let Err(e) = tch::vision::image::save(&thumbnail, &thumbnail_path) {
                    log::error!(
                        "[{}][{}/{}] failed to save thumbnail to file - {e}",
                        self.request.id,
                        idx,
                        self.request.num_samples
                    );
                }
            }
            Err(e) => {
                log::error!(
                    "[{}][{}/{}] failed to generate thumbnail - {e}",
                    self.request.id,
                    idx,
                    self.request.num_samples
                );
            }
        }
        if let Err(e) = self.tx_results.try_send(SaveImageFsResult {
            id: self.request.id.clone(),
            n_sample: idx as i32,
            is_last: idx == self.request.num_samples,
            path,
            thumbnail: thumbnail_path,
        }) {
            log::error!(
                "[{}][{}/{}] failed to send save request - {e}",
                self.request.id,
                idx,
                self.request.num_samples
            );
        }
        self.processed_timesteps = 0;
        self.processed_samples += 1;
    }

    pub fn decode_and_save_image(&mut self, latents: &Tensor) {
        let image = self.decode_latents(latents);
        self.save_image(image);
    }

    pub fn log_timestep(&self, type_: &'static str) {
        self.log(
            Level::Debug,
            type_,
            format!(
                "timestep {}/{}",
                self.processed_timesteps, self.request.n_steps,
            ),
        );
    }

    pub fn log(&self, level: Level, type_: &'static str, msg: impl std::fmt::Display) {
        log::log!(
            level,
            "[{type_}][{}][{}/{}] {msg}",
            self.request.id,
            self.processed_samples + 1,
            self.request.num_samples,
        )
    }
}

fn generate_thumbnail(
    image: &Tensor,
    width: usize,
    height: usize,
) -> std::result::Result<Tensor, tch::TchError> {
    tch::vision::image::resize(image, width as i64, height as i64)
}
