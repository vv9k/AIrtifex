use futures::channel::oneshot;
use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{Event, FileReader, HtmlInputElement};

pub fn extract_file_from_html_input(event: Event) -> Option<web_sys::File> {
    // Get the event target
    let target = event.target()?;

    // Cast the target to an HTMLInputElement
    let input_element = target.dyn_into::<HtmlInputElement>().ok()?;

    // Access the input element's files property
    let files = input_element.files()?;

    // Get the first file in the FileList
    files.get(0)
}

pub async fn read_file(file: web_sys::File) -> Result<Vec<u8>, JsValue> {
    let (tx, rx) = oneshot::channel::<Result<Vec<u8>, JsValue>>();

    let file_reader = FileReader::new()?;
    let reader = file_reader.clone();
    let onloadend_cb = Closure::once(move || {
        let result = match reader.result() {
            Ok(val) => val,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };
        let array = js_sys::Uint8Array::new(&result);
        let data: Vec<u8> = array.to_vec();
        let _ = tx.send(Ok(data));
    });

    file_reader.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
    file_reader.read_as_array_buffer(&file)?;

    // Await for the file to be read
    rx.await.map_err(|e| JsValue::from_str(&e.to_string()))?
}

pub fn sleep_promise(ms: i32) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    })
}

pub fn sleep(ms: i32) -> wasm_bindgen_futures::JsFuture {
    wasm_bindgen_futures::JsFuture::from(sleep_promise(ms))
}

/// Encodes image so that it can be used in an <img src=> tag
pub fn encode_image_base64(image: &[u8]) -> String {
    use base64::engine::Engine;
    let engine = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    );
    let encoded = engine.encode(image);
    format!("data:image/png;base64,{encoded}")
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, Debug)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

impl WindowSize {
    pub fn new() -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("Failed to get window object")?;
        let width = window
            .inner_width()
            .map(|w| w.as_f64())?
            .ok_or("failed to get window inner width")? as u32;
        let height = window
            .inner_height()
            .map(|h| h.as_f64())?
            .ok_or("failed to get window inner height")? as u32;

        Ok(Self { width, height })
    }

    pub fn signal(cx: Scope) -> Result<ReadSignal<WindowSize>, JsValue> {
        let (size_r, size_w) = create_signal(cx, Self::new()?);
        let mut should_write = true;

        let closure = Closure::wrap(Box::new(move |_event: Event| {
            let update_size = |s: &mut Self| {
                let size = Self::new().expect("window size");
                s.width = size.width;
                s.height = size.height;
            };
            if should_write && size_w.try_update(update_size).is_none() {
                should_write = false;
            }
        }) as Box<dyn FnMut(_)>);

        let window = web_sys::window().ok_or("Failed to get window object")?;
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(size_r)
    }
}

pub fn display_limited_str(s: &str, limit: usize) -> String {
    if s.len() > limit {
        format!("{}...", s.chars().take(limit).collect::<String>())
    } else {
        s.to_string()
    }
}

pub fn get_resolved_path(cx: Scope) -> String {
    let location = use_location(cx);
    location.pathname.get()
}
