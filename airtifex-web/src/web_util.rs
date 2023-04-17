use futures::channel::oneshot;
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

    let file_reader = FileReader::new().unwrap();
    let reader = file_reader.clone();
    let onloadend_cb = Closure::once(move || {
        let result = match reader.result() {
            Ok(val) => val,
            Err(e) => {
                let _ = tx.send(Err(e.into()));
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
