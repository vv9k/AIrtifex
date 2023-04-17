use crate::config::StableDiffusionConfig;
use crate::gen::image::sd::generator::{BaseImageGenerator, GenImageError, ImageGenerator};
use crate::gen::image::{InpaintData, SaveImageFsResult};
use crate::Result;

use diffusers::models::vae::DiagonalGaussianDistribution;
use flume::Sender;
use std::path::Path;
use tch::{Device, Kind, Tensor};

use super::LATENTS_SCALE;

pub struct InpaintImageGenerator {
    base_generator: BaseImageGenerator,
    latents: Tensor,
    mask: Tensor,
    masked_image_latents: Tensor,
    masked_image_dist: DiagonalGaussianDistribution,
}

impl InpaintImageGenerator {
    pub fn new(
        request: InpaintData,
        config: &StableDiffusionConfig,
        clip_device: Device,
        unet_device: Device,
        vae_device: Device,
        tx_results: Sender<SaveImageFsResult>,
        save_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let InpaintData {
            data,
            input_image,
            mask,
        } = request;
        let base_generator = BaseImageGenerator::new(
            data,
            config,
            clip_device,
            unet_device,
            vae_device,
            tx_results,
            save_dir,
            1,
        )?;

        let (mask, masked_image) = prepare_mask_and_masked_image(&input_image, &mask)?;
        let mask = mask.upsample_nearest2d(
            &[
                base_generator.sd_config.height / 8,
                base_generator.sd_config.width / 8,
            ],
            None,
            None,
        );
        let mask = Tensor::cat(&[&mask, &mask], 0).to_device(unet_device);
        let masked_image_dist = base_generator
            .vae
            .encode(&masked_image.to_device(vae_device));

        let mut g = Self {
            base_generator,
            latents: Tensor::new(),
            mask,
            masked_image_latents: Tensor::new(),
            masked_image_dist,
        };

        g.init_latents(0);

        Ok(g)
    }

    pub fn init_latents(&mut self, idx: i64) {
        tch::manual_seed(self.base_generator.request.seed + idx);
        let masked_image_latents =
            (self.masked_image_dist.sample() * LATENTS_SCALE).to(self.base_generator.unet_device);
        self.masked_image_latents = Tensor::cat(&[&masked_image_latents, &masked_image_latents], 0);
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

impl ImageGenerator for InpaintImageGenerator {
    fn is_finished(&self) -> bool {
        self.base_generator.is_finished()
    }
    fn type_(&self) -> &'static str {
        "inpaint"
    }

    fn base_generator(&self) -> &BaseImageGenerator {
        &self.base_generator
    }

    fn process_next_timestep(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }
        let _no_grad_guard = tch::no_grad_guard();
        let idx = (self.base_generator.processed_samples + 1) as i64;

        self.log_timestep();

        if self.base_generator.processed_timesteps == self.base_generator.request.n_steps {
            self.log(log::Level::Debug, "generating image");
            self.latents = self.latents.to(self.base_generator.vae_device);
            self.base_generator
                .decode_and_save_image(&self.latents, idx);

            if self.base_generator.processed_samples
                == self.base_generator.request.num_samples as usize
            {
                return true;
            }

            self.init_latents(idx);
        }

        let Some(&timestep) = self.base_generator.scheduler.timesteps().get(self.base_generator.processed_timesteps) else {
            return false
        };

        let latent_model_input = Tensor::cat(&[&self.latents, &self.latents], 0);
        self.log(log::Level::Debug, "got latent_model_input");

        let latent_model_input = self
            .base_generator
            .scheduler
            .scale_model_input(latent_model_input, timestep);
        self.log(log::Level::Debug, "got scaled latent_model_input");
        let latent_model_input = Tensor::cat(
            &[&latent_model_input, &self.mask, &self.masked_image_latents],
            1,
        );
        self.log(log::Level::Debug, "got concatenated latent_model_input");
        let noise_pred = self.base_generator.unet.forward(
            &latent_model_input,
            timestep as f64,
            &self.base_generator.text_embeddings,
        );
        self.log(log::Level::Debug, "got noise_pred");
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

fn prepare_mask_and_masked_image(input_image: &[u8], mask: &[u8]) -> Result<(Tensor, Tensor)> {
    let image =
        tch::vision::image::load_from_memory(input_image).map_err(GenImageError::LoadInputImage)?;
    let image = image / 255. * 2. - 1.;

    let mask = tch::vision::image::load_from_memory(mask).map_err(GenImageError::LoadMask)?;
    let mask = mask.mean_dim(Some([0].as_slice()), true, Kind::Float);
    let mask = mask.ge(122.5).totype(Kind::Float);
    let masked_image: Tensor = image * (1 - &mask);
    Ok((mask.unsqueeze(0), masked_image.unsqueeze(0)))
}
