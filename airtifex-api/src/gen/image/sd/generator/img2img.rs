use crate::config::StableDiffusionConfig;
use crate::gen::image::sd::generator::{BaseImageGenerator, ImageGenerator};
use crate::gen::image::{ImageToImageData, SaveImageFsResult};
use crate::Result;

use diffusers::models::vae::DiagonalGaussianDistribution;
use flume::Sender;
use std::path::Path;
use tch::{Device, Tensor};

use super::{GenImageError, LATENTS_SCALE};

pub struct ImageToImageGenerator {
    base_generator: BaseImageGenerator,
    init_latent_dist: DiagonalGaussianDistribution,
    latents: Tensor,
    t_start: usize,
}

impl ImageToImageGenerator {
    pub fn new(
        request: ImageToImageData,
        config: &StableDiffusionConfig,
        clip_device: Device,
        unet_device: Device,
        vae_device: Device,
        tx_results: Sender<SaveImageFsResult>,
        save_dir: impl AsRef<Path>,
    ) -> Result<Self> {
        let ImageToImageData { data, input_image } = request;
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
        let init_image = image_preprocess(&input_image[..])?;
        let init_image = init_image.to(vae_device);
        let init_latent_dist = base_generator.vae.encode(&init_image);

        let strength = 0.7; // TODO
        let t_start = base_generator.request.n_steps
            - (base_generator.request.n_steps as f64 * strength) as usize;

        let mut g = Self {
            base_generator,
            init_latent_dist,
            latents: Tensor::new(),
            t_start,
        };

        g.init_latents(0);

        Ok(g)
    }

    pub fn init_latents(&mut self, idx: i64) {
        tch::manual_seed(self.base_generator.request.seed + idx);
        let latents =
            (self.init_latent_dist.sample() * LATENTS_SCALE).to(self.base_generator.unet_device);
        let noise = latents.randn_like();
        self.latents = self.base_generator.scheduler.add_noise(
            &latents,
            noise,
            self.base_generator.scheduler.timesteps()[self.t_start],
        );
    }
}

impl ImageGenerator for ImageToImageGenerator {
    fn is_finished(&self) -> bool {
        self.base_generator.is_finished()
    }

    fn base_generator(&self) -> &BaseImageGenerator {
        &self.base_generator
    }

    fn type_(&self) -> &'static str {
        "img2img"
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

fn image_preprocess(input_image: &[u8]) -> Result<Tensor> {
    let image =
        tch::vision::image::load_from_memory(input_image).map_err(GenImageError::LoadInputImage)?;
    let (_num_channels, height, width) = image.size3().map_err(GenImageError::Get3dSize)?;
    let height = height - height % 32;
    let width = width - width % 32;
    let image =
        tch::vision::image::resize(&image, width, height).map_err(GenImageError::ImageResize)?;
    Ok((image / 255. * 2. - 1.).unsqueeze(0))
}
