use crate::config::{StableDiffusionConfig, StableDiffusionVersion};
use crate::gen::image::{SaveImageFsResult, TextToImageData};
use crate::Result;

use diffusers::models::unet_2d::UNet2DConditionModel;
use diffusers::models::vae::AutoEncoderKL;
use diffusers::pipelines::stable_diffusion;
use diffusers::schedulers::ddim::DDIMScheduler;
use diffusers::transformers::clip;
use flume::Sender;
use std::path::{Path, PathBuf};
use tch::{nn::Module, Device, Kind, Tensor};

#[derive(Debug, thiserror::Error)]
pub enum TextToImageError {
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
}

pub struct TextToImageGenerator {
    vae: AutoEncoderKL,
    vae_device: Device,
    unet: UNet2DConditionModel,
    unet_device: Device,
    scheduler: DDIMScheduler,
    text_embeddings: Tensor,
    tx_results: Sender<SaveImageFsResult>,
    data: TextToImageData,
    sd_config: stable_diffusion::StableDiffusionConfig,
    save_dir: PathBuf,
    processed_samples: usize,
    processed_timesteps: usize,
    latents: Tensor,
}

impl TextToImageGenerator {
    pub fn new(
        data: TextToImageData,
        config: &StableDiffusionConfig,
        clip_device: Device,
        unet_device: Device,
        vae_device: Device,
        tx_results: Sender<SaveImageFsResult>,
        save_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let sd_config = match config.version {
            StableDiffusionVersion::V2_1 => stable_diffusion::StableDiffusionConfig::v2_1(
                None,
                Some(data.height),
                Some(data.width),
            ),
            _ => stable_diffusion::StableDiffusionConfig::v1_5(
                None,
                Some(data.height),
                Some(data.width),
            ),
        };
        let scheduler = sd_config.build_scheduler(data.n_steps);
        let tokenizer = clip::Tokenizer::create(&config.vocab_file, &sd_config.clip)
            .map_err(TextToImageError::ClipTokenizerInit)?;

        log::debug!(
            "[{}] Generating image for prompt \"{}\".",
            data.id,
            data.prompt
        );
        let tokens = tokenizer
            .encode(&data.prompt)
            .map_err(TextToImageError::PromptEncode)?;

        let tokens: Vec<i64> = tokens.into_iter().map(|x| x as i64).collect();
        let tokens = Tensor::of_slice(&tokens).view((1, -1)).to(clip_device);
        let uncond_tokens = tokenizer
            .encode("")
            .map_err(TextToImageError::TokensEncode)?;
        let uncond_tokens: Vec<i64> = uncond_tokens.into_iter().map(|x| x as i64).collect();
        let uncond_tokens = Tensor::of_slice(&uncond_tokens)
            .view((1, -1))
            .to(clip_device);

        let no_grad_guard = tch::no_grad_guard();

        log::debug!("[{}] Building the Clip transformer.", data.id);
        let text_model = sd_config
            .build_clip_transformer(&config.clip_weights_path.to_string_lossy(), clip_device)
            .map_err(TextToImageError::ClipTransformerBuild)?;

        let text_embeddings = text_model.forward(&tokens);
        let uncond_embeddings = text_model.forward(&uncond_tokens);
        let text_embeddings = Tensor::cat(&[uncond_embeddings, text_embeddings], 0).to(unet_device);

        log::debug!("[{}] Building the autoencoder", data.id);
        let vae = sd_config
            .build_vae(&config.vae_weights_path.to_string_lossy(), vae_device)
            .map_err(TextToImageError::VaeBuild)?;

        log::debug!("[{}] Building unet", data.id);
        let unet = sd_config
            .build_unet(&config.unet_weights_path.to_string_lossy(), unet_device, 4)
            .map_err(TextToImageError::UnetBuild)?;

        tch::manual_seed(data.seed);
        let latents = Tensor::randn(
            &[1, 4, sd_config.height / 8, sd_config.width / 8],
            (Kind::Float, unet_device),
        );

        drop(no_grad_guard);

        Ok(Self {
            vae,
            vae_device,
            unet,
            unet_device,
            scheduler,
            text_embeddings,
            tx_results,
            data,
            sd_config,
            save_dir: save_dir.as_ref().to_path_buf(),
            processed_samples: 0,
            processed_timesteps: 0,
            latents,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.processed_samples as i64 >= self.data.num_samples
    }

    pub fn process_next_timestep(&mut self, bsize: i64) -> bool {
        if self.is_finished() {
            return false;
        }
        let _no_grad_guard = tch::no_grad_guard();
        let idx = (self.processed_samples + 1) as i64;
        let id = &self.data.id;
        let n_samples = self.data.num_samples;

        log::debug!(
            "[{id}][{idx}/{n_samples}] timestep {}/{}",
            self.processed_timesteps,
            self.data.n_steps,
        );

        if self.processed_timesteps == self.data.n_steps {
            log::debug!("[{id}][{idx}/{n_samples}] generating image",);
            self.latents = self.latents.to(self.vae_device);
            let image = self.vae.decode(&(&self.latents / 0.18215));
            let image = (image / 2 + 0.5).clamp(0., 1.).to_device(Device::Cpu);
            let image = (image * 255.).to_kind(Kind::Uint8);

            self.save_image(image, idx);

            self.processed_timesteps = 0;
            self.processed_samples += 1;

            if self.processed_samples == self.data.num_samples as usize {
                return true;
            }

            tch::manual_seed(self.data.seed + idx);
            self.latents = Tensor::randn(
                &[
                    bsize,
                    4,
                    self.sd_config.height / 8,
                    self.sd_config.width / 8,
                ],
                (Kind::Float, self.unet_device),
            );
        }

        let Some(&timestep) = self.scheduler.timesteps().get(self.processed_timesteps) else {
            return false
        };

        let latent_model_input = Tensor::cat(&[&self.latents, &self.latents], 0);

        let latent_model_input = self
            .scheduler
            .scale_model_input(latent_model_input, timestep);
        let noise_pred =
            self.unet
                .forward(&latent_model_input, timestep as f64, &self.text_embeddings);
        let noise_pred = noise_pred.chunk(2, 0);
        let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
        let noise_pred =
            noise_pred_uncond + (noise_pred_text - noise_pred_uncond) * self.data.guidance_scale;
        self.latents = self.scheduler.step(&noise_pred, timestep, &self.latents);

        self.processed_timesteps += 1;

        true
    }

    fn save_image(&self, image: Tensor, idx: i64) {
        let path = self.save_dir.join(format!("{}-{idx}.png", self.data.id));
        if let Err(e) = tch::vision::image::save(&image, &path) {
            log::error!(
                "[{}][{}/{}] failed to save image to file - {e}",
                self.data.id,
                idx,
                self.data.num_samples
            );
        }
        if let Err(e) = self.tx_results.try_send(SaveImageFsResult {
            id: self.data.id.clone(),
            n_sample: idx as i32,
            is_last: idx == self.data.num_samples,
            path,
        }) {
            log::error!(
                "[{}][{}/{}] failed to send save request - {e}",
                self.data.id,
                idx,
                self.data.num_samples
            );
        }
    }
}
