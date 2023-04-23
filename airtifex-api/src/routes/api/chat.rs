use crate::{
    auth::Claims,
    gen::llm::{ChatData, InferenceRequest},
    id::Uuid,
    models::{chat::Chat, chat_entry::ChatEntry, llm::LargeLanguageModel},
    routes::handle_db_result_as_json,
    Error, SharedAppState, ToAxumResponse,
};
use airtifex_core::{
    api_response::ApiResponse,
    llm::{
        ChatEntryListEntry, ChatListEntry, ChatResponseRequest, ChatStartRequest,
        ChatStartResponse, ChatStreamResult, InferenceSettings, LlmListEntry,
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
        .route("/chat", routing::post(start_chat).get(list))
        .route("/chat/counters", routing::get(counters))
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
        user: claims.sub,
        save: true,
        chat_data: Some(ChatData {
            conversation_id: id,
            history,
        }),
        prompt: request.prompt,
        settings: InferenceSettings {
            num_predict: chat.num_predict.map(|k| k as usize),
            system_prompt: chat.system_prompt,
            n_batch: chat.n_batch.map(|k| k as usize),
            top_k: chat.top_k.map(|k| k as usize),
            top_p: chat.top_p,
            repeat_penalty: chat.repeat_penalty,
            temp: chat.temp,
        },
        play_back_tokens: false,
    };
    log::info!("{request:?}");

    if let Some((_, model)) = state.tx_inference_req.get(&chat.model) {
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

    let mut chat = Chat::new(claims.sub, model.clone(), request.title, request.settings);

    if let Some((config, _)) = state.tx_inference_req.get(&model) {
        if chat.n_batch.is_none() {
            chat.n_batch = Some(config.batch_size as i32);
        }
        if chat.top_k.is_none() {
            chat.top_k = Some(config.top_k as i32);
        }
        if chat.top_p.is_none() {
            chat.top_p = Some(config.top_p);
        }
        if chat.repeat_penalty.is_none() {
            chat.repeat_penalty = Some(config.repeat_penalty);
        }
        if chat.temp.is_none() {
            chat.temp = Some(config.temperature);
        }
    }

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
                        settings: airtifex_core::llm::InferenceSettings {
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
                settings: airtifex_core::llm::InferenceSettings {
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

async fn counters(claims: Claims, State(state): State<SharedAppState>) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);

    handle_db_result_as_json(Chat::counters(&db, &claims.sub).await.map_err(Error::from))
}
