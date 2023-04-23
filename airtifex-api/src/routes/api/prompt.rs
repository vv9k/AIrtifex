use crate::{
    auth::Claims, gen::llm::InferenceRequest, id::Uuid, models::prompt::Prompt,
    routes::handle_db_result_as_json, Error, SharedAppState, ToAxumResponse,
};
use airtifex_core::{
    api_response::ApiResponse,
    llm::{ChatStreamResult, InferenceSettings, OneshotInferenceRequest, PromptInspect},
};

use axum::{
    body::StreamBody,
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};

pub fn router() -> Router<SharedAppState> {
    Router::new()
        .route("/inference", routing::post(oneshot_inference))
        .route("/prompt", routing::get(list))
        .route(
            "/prompt/:id",
            routing::get(get_prompt).delete(delete_prompt),
        )
}

async fn oneshot_inference(
    claims: Claims,
    State(state): State<SharedAppState>,
    Json(request): Json<OneshotInferenceRequest>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    let (tx_tokens, rx_tokens): (
        flume::Sender<ChatStreamResult>,
        flume::Receiver<ChatStreamResult>,
    ) = flume::unbounded();

    let inference_request = InferenceRequest {
        tx_tokens,
        save: request.save,
        user: claims.sub,
        chat_data: None,
        prompt: request.prompt,
        settings: InferenceSettings {
            num_predict: request.num_predict,
            system_prompt: None,
            n_batch: request.n_batch,
            top_k: request.top_k,
            top_p: request.top_p,
            repeat_penalty: request.repeat_penalty,
            temp: request.temp,
        },
        play_back_tokens: request.play_back_tokens,
    };
    log::info!("{inference_request:?}");

    if let Some((_, model)) = state.tx_inference_req.get(&request.model) {
        if let Err(e) = model.send_async(inference_request).await {
            return ApiResponse::failure(e).internal_server_error();
        }
    } else {
        return ApiResponse::failure(format!("failed to find model {}", &request.model))
            .internal_server_error();
    }

    (
        [
            (axum::http::header::CONTENT_TYPE, "text/event-stream"),
            (axum::http::header::TRANSFER_ENCODING, "chunked"),
        ],
        StreamBody::new(rx_tokens.into_stream()),
    )
        .into_response()
}

async fn list(claims: Claims, State(state): State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Prompt::list_prompts_of_user(&db, &claims.sub)
            .await
            .map(|p| {
                p.into_iter()
                    .map(|p| PromptInspect {
                        id: p.id.to_string(),
                        prompt: p.prompt,
                        date: p.date,
                        username: p.username,
                        response: p.response,
                        model: p.model,
                        n_batch: p.n_batch.map(|v| v as usize),
                        num_predict: p.num_predict.map(|v| v as usize),
                        top_k: p.top_k.map(|v| v as usize),
                        top_p: p.top_p,
                        repeat_penalty: p.repeat_penalty,
                        temp: p.temp,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn get_prompt(
    claims: Claims,
    State(state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Prompt::get_prompt_for_user(&db, &claims.sub, &id)
            .await
            .map(|p| PromptInspect {
                id: p.id.to_string(),
                prompt: p.prompt,
                date: p.date,
                username: p.username,
                response: p.response,
                model: p.model,
                n_batch: p.n_batch.map(|v| v as usize),
                num_predict: p.num_predict.map(|v| v as usize),
                top_k: p.top_k.map(|v| v as usize),
                top_p: p.top_p,
                repeat_penalty: p.repeat_penalty,
                temp: p.temp,
            })
            .map_err(Error::from),
    )
}

async fn delete_prompt(
    claims: Claims,
    State(state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Prompt::delete_prompt_for_user(&db, &claims.sub, &id)
            .await
            .map_err(Error::from),
    )
}
