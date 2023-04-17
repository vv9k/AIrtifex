use crate::config::StableDiffusionConfig;
use crate::gen::image::sd::generator::{BaseImageGenerator, ImageGenerator};
use crate::gen::image::{BaseImageData, SaveImageFsResult};
use crate::Result;

use flume::Sender;
use std::path::Path;
use tch::{Device, Kind, Tensor};

pub struct TextToImageGenerator {
    base_generator: BaseImageGenerator,
    latents: Tensor,
}

impl TextToImageGenerator {
    pub fn new(
        request: BaseImageData,
        config: &StableDiffusionConfig,
        clip_device: Device,
        unet_device: Device,
        vae_device: Device,
        tx_results: Sender<SaveImageFsResult>,
        save_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let base_generator = BaseImageGenerator::new(
            request,
            config,
            clip_device,
            unet_device,
            vae_device,
            tx_results,
            save_dir,
            1,
        )?;

        let mut g = Self {
            base_generator,
            latents: Tensor::new(),
        };

        g.init_latents();

        Ok(g)
    }

    pub fn init_latents(&mut self) {
        tch::manual_seed(self.base_generator.request.seed + self.base_generator.sample_idx());
        self.latents = Tensor::randn(
            &[
                self.base_generator.bsize,
                4,
                self.base_generator.sd_config.height / 8,
                self.base_generator.sd_config.width / 8,
            ],
            (Kind::Float, self.base_generator.unet_device),
        );
        self.latents *= self.base_generator.scheduler.init_noise_sigma();
    }
}

impl ImageGenerator for TextToImageGenerator {
    fn is_finished(&self) -> bool {
        self.base_generator.is_finished()
    }

    fn type_(&self) -> &'static str {
        "text2img"
    }

    fn base_generator(&self) -> &BaseImageGenerator {
        &self.base_generator
    }

    fn process_next_timestep(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }
        let _no_grad_guard = tch::no_grad_guard();

        self.log_timestep();

        if self.base_generator.processed_timesteps == self.base_generator.request.n_steps {
            self.log(log::Level::Debug, "generating image");
            self.latents = self.latents.to(self.base_generator.vae_device);
            self.base_generator.decode_and_save_image(&self.latents);

            if self.base_generator.processed_samples
                == self.base_generator.request.num_samples as usize
            {
                return true;
            }

            self.init_latents();
        }

        let Some(&timestep) = self.base_generator.scheduler.timesteps().get(self.base_generator.processed_timesteps) else {
            return false
        };

        let latent_model_input = Tensor::cat(&[&self.latents, &self.latents], 0);

        let latent_model_input = self
            .base_generator
            .scheduler
            .scale_model_input(latent_model_input, timestep);
        let noise_pred = self.base_generator.unet.forward(
            &latent_model_input,
            timestep as f64,
            &self.base_generator.text_embeddings,
        );
        let noise_pred = noise_pred.chunk(2, 0);
        let (noise_pred_uncond, noise_pred_text) = (&noise_pred[0], &noise_pred[1]);
        let noise_pred = noise_pred_uncond
            + (noise_pred_text - noise_pred_uncond) * self.base_generator.request.guidance_scale;
        self.latents = self
            .base_generator
            .scheduler
            .step(&noise_pred, timestep, &self.latents);

        self.base_generator.processed_timesteps += 1;

        true
    }
}
