mod generator;

pub use generator::GenImageError;

use crate::config::StableDiffusionConfig;
use crate::gen::image::{GenerateImageRequest, SaveImageFsResult};
use crate::models::image::Image;
use crate::models::image_sample::ImageSample;
use crate::queue;
use generator::img2img::ImageToImageGenerator;
use generator::inpaint::InpaintImageGenerator;
use generator::txt2img::TextToImageGenerator;

use flume::{unbounded, Receiver, Sender};
use std::sync::Arc;
use tokio::runtime::Runtime;

use self::generator::ImageGenerator;

pub fn initialize(
    db: Arc<crate::DbPool>,
    config: StableDiffusionConfig,
    runtime: Arc<Runtime>,
) -> Sender<GenerateImageRequest> {
    let request_queue = queue::empty_queue();

    // Create a channel and thread responsible for saving images to database
    let (tx_results, rx_results): (Sender<SaveImageFsResult>, Receiver<SaveImageFsResult>) =
        unbounded();
    std::thread::spawn(move || loop {
        if let Ok(save_data_request) = rx_results.recv() {
            let db = db.clone();
            if let Ok(data) = std::fs::read(&save_data_request.path) {
                let _ = runtime.spawn(async move {
                    log::debug!(
                        "[{}][{}] saving image to DB",
                        save_data_request.id,
                        save_data_request.n_sample
                    );
                    let entry = ImageSample::new(
                        save_data_request.id.parse().unwrap(),
                        save_data_request.n_sample,
                        data,
                    );
                    if let Err(e) = entry.create(&db).await {
                        log::error!(
                            "[{}][{}] failed to save image data- {e}",
                            save_data_request.id,
                            save_data_request.n_sample
                        )
                    }
                    if save_data_request.is_last {
                        log::debug!(
                            "[{}][{}] updating image processing to false",
                            save_data_request.id,
                            save_data_request.n_sample
                        );
                        if let Ok(id) = save_data_request.id.parse() {
                            if let Err(e) = Image::update_is_processing(&db, &id, false).await {
                                log::error!(
                                    "[{}][{}] failed to update image processing status - {e}",
                                    save_data_request.id,
                                    save_data_request.n_sample
                                )
                            }
                        }
                    }
                });
            }
        } else {
            log::error!("all channels closed");
            break;
        }
    });

    // Create a thread that'll receive GenerateImageRequest
    let queue = request_queue.clone();
    let tx_request = queue::start_queue_thread::<GenerateImageRequest>(queue);

    // Create a thread that will handle generating images
    std::thread::spawn(move || {
        let cpu = {
            let mut cpu = vec![];
            if config.clip_cpu {
                cpu.push("clip".into())
            }
            if config.vae_cpu {
                cpu.push("vae".into())
            }
            if config.unet_cpu {
                cpu.push("unet".into())
            }
            cpu
        };
        let device_setup = diffusers::utils::DeviceSetup::new(cpu);

        let clip_device = device_setup.get("clip");
        let vae_device = device_setup.get("vae");
        let unet_device = device_setup.get("unet");

        let tmp = tempfile::TempDir::new().expect("temporary directory for images");

        let mut running_sessions = Vec::new();

        loop {
            let free_spots = config.max_image_gen_sessions - running_sessions.len();

            if free_spots > 0 {
                if let Ok(mut queue) = request_queue.try_write() {
                    'inner: while let Some(request) = queue.pop_front() {
                        let id = request.id().to_string();
                        let generator = match request {
                            GenerateImageRequest::ImageToImage(data) => {
                                if config.feature_image_to_image {
                                    match ImageToImageGenerator::new(
                                        data,
                                        &config,
                                        clip_device.clone(),
                                        unet_device.clone(),
                                        vae_device.clone(),
                                        tx_results.clone(),
                                        tmp.path(),
                                    ) {
                                        Ok(generator) => {
                                            Box::new(generator) as Box<dyn ImageGenerator>
                                        }
                                        Err(e) => {
                                            log::error!("[{id}] {e}");
                                            continue 'inner;
                                        }
                                    }
                                } else {
                                    log::error!("[{id}] feature image-to-image is disabled");
                                    continue 'inner;
                                    // # TODO return an error somehow
                                }
                            }
                            GenerateImageRequest::Inpaint(data) => {
                                if config.feature_inpaint {
                                    match InpaintImageGenerator::new(
                                        data,
                                        &config,
                                        clip_device.clone(),
                                        unet_device.clone(),
                                        vae_device.clone(),
                                        tx_results.clone(),
                                        tmp.path(),
                                    ) {
                                        Ok(generator) => {
                                            Box::new(generator) as Box<dyn ImageGenerator>
                                        }
                                        Err(e) => {
                                            log::error!("[{id}] {e}");
                                            continue;
                                        }
                                    }
                                } else {
                                    log::error!(
                                        "[{id}] feature inpaint is disabled for this model"
                                    );
                                    continue 'inner;
                                    // # TODO return an error somehow
                                }
                            }
                            GenerateImageRequest::TextToImage(data) => {
                                if config.feature_text_to_image {
                                    match TextToImageGenerator::new(
                                        data,
                                        &config,
                                        clip_device.clone(),
                                        unet_device.clone(),
                                        vae_device.clone(),
                                        tx_results.clone(),
                                        tmp.path(),
                                    ) {
                                        Ok(generator) => {
                                            Box::new(generator) as Box<dyn ImageGenerator>
                                        }
                                        Err(e) => {
                                            log::error!("[{id}] {e}");
                                            continue;
                                        }
                                    }
                                } else {
                                    log::error!(
                                        "[{id}] feature text-to-image is disabled for this model"
                                    );
                                    continue 'inner;
                                    // # TODO return an error somehow
                                }
                            }
                        };
                        running_sessions.push(generator);

                        if free_spots == 0 {
                            break 'inner;
                        }
                    }
                }
            }

            for session in &mut running_sessions {
                session.process_next_timestep();
            }

            running_sessions.retain(|s| !s.is_finished());

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    tx_request
}
