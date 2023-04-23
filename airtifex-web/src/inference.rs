use crate::api;
use crate::components::status_message::Message;

use futures::StreamExt;
use leptos::*;
use wasm_bindgen::JsCast;

pub async fn read_inference_stream(
    cx: Scope,
    resp: Result<gloo_net::http::Response, api::Error>,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    response_view: RwSignal<String>,
    status_message: RwSignal<Message>,
    should_cancel: RwSignal<bool>,
) {
    match resp {
        Ok(response) => {
            if let Some(body) = response.body() {
                let body = body.unchecked_into::<wasm_streams::readable::sys::ReadableStream>();
                let body = wasm_streams::ReadableStream::from_raw(body);
                let mut reader = body.into_stream();

                response_view.update(|rsp| *rsp = "".into());

                loop {
                    if should_cancel.get() {
                        should_cancel.update(|c| *c = false);
                        break;
                    }
                    match reader.next().await {
                        Some(Ok(chunk)) => {
                            let array: Vec<_> = js_sys::Array::from(&chunk)
                                .iter()
                                .map(|v| v.as_f64().unwrap_or_default() as u8)
                                .collect();
                            let token = String::from_utf8_lossy(&array[..]);
                            response_view.update(|rsp| {
                                rsp.push_str(&token);
                            });
                        }
                        Some(Err(e)) => {
                            status_message
                                .update(|m| *m = Message::Error(e.as_string().unwrap_or_default()));
                            break;
                        }
                        None => break,
                    }
                }
            } else {
                status_message.update(|m| {
                    *m = Message::Error("response body empty".into());
                });
            }
        }
        Err(err) => {
            let e = err.to_string();
            crate::pages::goto_login_if_expired(cx, &e, authorized_api);
            status_message.update(|m| {
                *m = Message::Error(format!("failed to generate an answer - {e}"));
            })
        }
    }
}
