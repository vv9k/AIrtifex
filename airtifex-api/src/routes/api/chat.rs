use crate::auth::Claims;
use crate::gen::llm::{ChatData, InferenceRequest};
use crate::id::Uuid;
use crate::models::{chat::Chat, chat_entry::ChatEntry, llm::LargeLanguageModel};
use crate::routes::handle_db_result_as_json;
use crate::Error;
use crate::{SharedAppState, ToAxumResponse};
use airtifex_core::{
    api_response::ApiResponse,
    llm::{
        ChatEntryListEntry, ChatListEntry, ChatResponseRequest, ChatStartRequest,
        ChatStartResponse, ChatStreamResult, LlmListEntry, OneshotInferenceRequest,
    },
};

use axum::{
    body::StreamBody,
    extract::{Json, Path, State},
    response::{IntoResponse, Response},
    routing, Router,
};

pub fn router() -> Router<SharedAppState> {
    Router::new()
        .route("/models", routing::get(list_models))
        .route("/inference", routing::post(oneshot_inference))
        .route("/chat", routing::post(start_chat).get(list))
        .route(
            "/chat/:id",
            routing::get(get_chat).delete(delete_chat).post(inference),
        )
        .route("/chat/:id/history", routing::get(get_chat_history))
}

async fn inference(
    claims: Claims,
    State(state): State<SharedAppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ChatResponseRequest>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    let (tx_tokens, rx_tokens): (
        flume::Sender<ChatStreamResult>,
        flume::Receiver<ChatStreamResult>,
    ) = flume::unbounded();

    let history = match Chat::list_entries(&db, &id, &claims.sub).await {
        Ok(chat) => chat,
        Err(e) => {
            return ApiResponse::failure(e).internal_server_error();
        }
    };

    let chat = match Chat::get_chat_for_user(&db, &claims.sub, &id).await {
        Ok(chat) => chat,
        Err(e) => {
            return ApiResponse::failure(e).internal_server_error();
        }
    };

    let request = InferenceRequest {
        tx_tokens,
        chat_data: Some(ChatData {
            user: claims.sub,
            conversation_id: id,
            history,
        }),
        num_predict: chat.num_predict.map(|k| k as usize),
        system_prompt: chat.system_prompt,
        prompt: request.prompt,
        n_batch: chat.n_batch.map(|k| k as usize),
        top_k: chat.top_k.map(|k| k as usize),
        top_p: chat.top_p,
        repeat_penalty: chat.repeat_penalty,
        temp: chat.temp,
        play_back_tokens: false,
    };
    log::info!("{request:?}");

    if let Some(model) = state.tx_inference_req.get(&chat.model) {
        if let Err(e) = model.send_async(request).await {
            return ApiResponse::failure(e).internal_server_error();
        }
    } else {
        return ApiResponse::failure(format!("failed to find model {}", &chat.model))
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

async fn start_chat(
    claims: Claims,
    State(state): State<SharedAppState>,
    Json(request): Json<ChatStartRequest>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    log::info!("{request:?}");
    let model = if let Some(model) = request.model {
        model
    } else {
        state
            .tx_inference_req
            .keys()
            .next()
            .map(|k| k.to_string())
            .unwrap_or_default()
    };

    let chat = Chat::new(claims.sub, model, request.title, request.settings);

    handle_db_result_as_json(
        chat.create(db)
            .await
            .map(|_| ChatStartResponse {
                chat_id: chat.id().to_string(),
            })
            .map_err(Error::from),
    )
}

async fn list(claims: Claims, state: State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Chat::list_chats_of_user(&db, &claims.sub)
            .await
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|chat| ChatListEntry {
                        id: chat.id.to_string(),
                        title: chat.title,
                        username: chat.username,
                        start_date: chat.start_date,
                        model: chat.model,
                        settings: airtifex_core::llm::ChatSettings {
                            num_predict: chat.num_predict.map(|n| n as usize),
                            system_prompt: chat.system_prompt,
                            n_batch: chat.n_batch.map(|n| n as usize),
                            top_k: chat.top_k.map(|n| n as usize),
                            top_p: chat.top_p,
                            repeat_penalty: chat.repeat_penalty,
                            temp: chat.temp,
                        },
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn get_chat(claims: Claims, state: State<SharedAppState>, Path(id): Path<Uuid>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        Chat::get_chat_for_user(&db, &claims.sub, &id)
            .await
            .map(|chat| ChatListEntry {
                id: chat.id.to_string(),
                title: chat.title,
                username: chat.username,
                start_date: chat.start_date,
                model: chat.model,
                settings: airtifex_core::llm::ChatSettings {
                    num_predict: chat.num_predict.map(|n| n as usize),
                    system_prompt: chat.system_prompt,
                    n_batch: chat.n_batch.map(|n| n as usize),
                    top_k: chat.top_k.map(|n| n as usize),
                    top_p: chat.top_p,
                    repeat_penalty: chat.repeat_penalty,
                    temp: chat.temp,
                },
            })
            .map_err(Error::from),
    )
}

async fn get_chat_history(
    claims: Claims,
    state: State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        ChatEntry::get_chat_entries(&db, &id, &claims.sub)
            .await
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|e| ChatEntryListEntry {
                        id: e.entry_id.to_string(),
                        chat_id: e.chat_id.to_string(),
                        content: e.content,
                        entry_type: e.entry_type,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn list_models(claims: Claims, state: State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(
        LargeLanguageModel::list(&db)
            .await
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|model| LlmListEntry {
                        model_id: model.model_id.to_string(),
                        name: model.name,
                        description: model.description,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(Error::from),
    )
}

async fn delete_chat(
    claims: Claims,
    state: State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(Chat::delete(&db, &id).await.map_err(Error::from))
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
        chat_data: None,
        num_predict: request.num_predict,
        system_prompt: None,
        prompt: request.prompt,
        n_batch: request.n_batch,
        top_k: request.top_k,
        top_p: request.top_p,
        repeat_penalty: request.repeat_penalty,
        temp: request.temp,
        play_back_tokens: request.play_back_tokens,
    };
    log::info!("{inference_request:?}");

    if let Some(model) = state.tx_inference_req.get(&request.model) {
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
