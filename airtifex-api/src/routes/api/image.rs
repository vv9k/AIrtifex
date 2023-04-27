use crate::{
    auth::Claims,
    gen::image::{BaseImageData, GenerateImageRequest, ImageToImageData, InpaintData},
    id::Uuid,
    models::{image::Image, image_model::ImageModel, image_sample::ImageSample, user::User},
    routes::handle_db_result_as_json,
    Error, SharedAppState, ToAxumResponse,
};
use airtifex_core::{
    api_response::ApiResponse,
    image::{
        ImageGenerateRequest, ImageInspect, ImageModelFeatures, ImageModelListEntry,
        ImageSampleInspect, TextToImageResponse,
    },
};

use axum::{
    extract::{Json, Path, State},
    response::Response,
    routing, Router,
};
use rand::Rng;

pub fn router() -> Router<SharedAppState> {
    Router::new()
        .route("/generate", routing::post(generate_image))
        .route("/", routing::get(list_images))
        .route("/models", routing::get(list_models))
        .route(
            "/:id",
            routing::get(get_image_metadata).delete(delete_image),
        )
        .route("/:id/samples", routing::get(list_image_entries))
        .route("/:id/samples/:n", routing::get(get_image_entry))
}

async fn generate_image(
    claims: Claims,
    State(state): State<SharedAppState>,
    Json(request): Json<ImageGenerateRequest>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    log::info!("{request:?}");

    let user_id = match User::get(db, &claims.sub).await.map(|u| u.id) {
        Ok(id) => id,
        Err(e) => return ApiResponse::failure(e).internal_server_error(),
    };

    let guidance_scale = request.guidance_scale.unwrap_or(7.5).min(20.0);
    let num_samples = request.num_samples.unwrap_or(1).min(16);
    let n_steps = request.n_steps.unwrap_or(25).min(420) as i64;

    let (data, mask, strength) = request
        .input_image
        .map(|i| (Some(i.data), i.mask, i.strength))
        .unwrap_or_default();

    let image = Image::new(
        user_id,
        request.model,
        request.width.unwrap_or(512),
        request.height.unwrap_or(512),
        request.prompt,
        data,
        mask,
        None,
        strength,
        n_steps,
        request.seed.unwrap_or_else(|| rand::thread_rng().gen()),
        num_samples,
        guidance_scale,
    );

    if let Err(e) = image.create(db).await {
        return ApiResponse::failure(e).internal_server_error();
    }

    let data = BaseImageData {
        id: image.id.to_string(),
        prompt: image.prompt,
        // input_image: image.input_image,
        // mask: image.mask,
        width: image.width,
        height: image.height,
        n_steps: image.n_steps as usize,
        seed: image.seed,
        num_samples: image.num_samples,
        guidance_scale: image.guidance_scale,
    };
    let request = match (image.input_image, image.mask) {
        (Some(input_image), Some(mask)) => GenerateImageRequest::Inpaint(InpaintData {
            data,
            input_image,
            mask,
        }),
        (Some(input_image), None) => GenerateImageRequest::ImageToImage(ImageToImageData {
            data,
            input_image,
            strength: image.strength.unwrap_or(0.7),
        }),
        (None, None) | (None, Some(_)) => GenerateImageRequest::TextToImage(data),
    };

    if let Some(tx_gen_req) = state.tx_image_gen_req.get(&image.model) {
        if let Err(e) = tx_gen_req.send_async(request).await {
            return ApiResponse::failure(e).internal_server_error();
        }
    } else {
        return ApiResponse::failure("Image generation from text is disabled")
            .internal_server_error();
    }

    ApiResponse::success(TextToImageResponse {
        image_id: image.id.to_string(),
    })
    .ok()
}

async fn list_images(claims: Claims, state: State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Image::list(db)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| ImageInspect {
                        id: e.id.to_string(),
                        user_id: e.user_id.to_string(),
                        model: e.model,
                        width: e.width,
                        height: e.height,
                        prompt: e.prompt,
                        input_image: e.input_image,
                        mask: e.mask,
                        thumbnail: e.thumbnail,
                        n_steps: e.n_steps,
                        seed: e.seed,
                        num_samples: e.num_samples,
                        processing: e.processing,
                        create_date: e.create_date,
                        guidance_scale: e.guidance_scale,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn list_image_entries(
    claims: Claims,
    state: State<SharedAppState>,
    id: Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        ImageSample::get_image_samples(db, &id)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| ImageSampleInspect {
                        sample_id: e.sample_id.to_string(),
                        image_id: e.image_id.to_string(),
                        n_sample: e.n,
                        data: e.data,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn get_image_entry(
    claims: Claims,
    state: State<SharedAppState>,
    Path((id, n)): Path<(Uuid, i32)>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        ImageSample::get_sample(db, &id, n)
            .await
            .map(|e| ImageSampleInspect {
                sample_id: e.sample_id.to_string(),
                image_id: e.image_id.to_string(),
                n_sample: e.n,
                data: e.data,
            })
            .map_err(Error::from),
    )
}

async fn get_image_metadata(
    claims: Claims,
    state: State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    let user_id = match User::get(db, &claims.sub).await.map(|u| u.id) {
        Ok(id) => id.to_string(),
        Err(e) => return ApiResponse::failure(e).internal_server_error(),
    };

    handle_db_result_as_json(
        Image::get_by_id(db, &id)
            .await
            .map(|image| ImageInspect {
                id: image.id.to_string(),
                user_id,
                model: image.model,
                width: image.width,
                height: image.height,
                prompt: image.prompt,
                input_image: image.input_image,
                mask: image.mask,
                thumbnail: image.thumbnail,
                n_steps: image.n_steps,
                seed: image.seed,
                num_samples: image.num_samples,
                processing: image.processing,
                create_date: image.create_date,
                guidance_scale: image.guidance_scale,
            })
            .map_err(Error::from),
    )
}

async fn delete_image(
    claims: Claims,
    state: State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(Image::delete(db, &id).await.map_err(Error::from))
}

async fn list_models(claims: Claims, state: State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        ImageModel::list(db)
            .await
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|model| ImageModelListEntry {
                        model_id: model.model_id.to_string(),
                        name: model.name,
                        description: model.description,
                        features: ImageModelFeatures {
                            inpaint: model.feature_inpaint,
                            text_to_image: model.feature_text_to_image,
                            image_to_image: model.feature_image_to_image,
                        },
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}
