pub mod history;
pub mod view;

pub use history::*;
pub use view::*;

use crate::components::{loading::*, status_message::*};
use crate::inference::read_inference_stream;
use crate::{api, web_util, Page, PageStack};
use airtifex_core::llm::OneshotInferenceRequest;

use leptos::*;

#[component]
pub fn PromptGenerate(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let status_message = create_rw_signal(cx, Message::Empty);

    let selected_model = create_rw_signal(cx, String::new());

    let num_predict = create_rw_signal(cx, None::<usize>);
    let prompt = create_rw_signal(cx, String::new());
    let n_batch = create_rw_signal(cx, None::<usize>);
    let top_k = create_rw_signal(cx, None::<usize>);
    let top_p = create_rw_signal(cx, None::<f32>);
    let repeat_penalty = create_rw_signal(cx, None::<f32>);
    let temp = create_rw_signal(cx, None::<f32>);
    let play_back_tokens = create_rw_signal(cx, true);
    let response_view = create_rw_signal(cx, String::new());

    let is_inference_running = create_rw_signal(cx, false);
    let should_cancel = create_rw_signal(cx, false);

    let window_size = web_util::WindowSize::signal(cx).expect("window size");

    let inference_action = create_action(cx, move |_| async move {
        if let Some(api) = authorized_api.get() {
            if is_inference_running.get() && !should_cancel.get() {
                should_cancel.update(|c| *c = true);
                // wait for any other job to cancel
                let _ = web_util::sleep(100).await;
            }
            is_inference_running.update(|r| *r = true);

            let request = OneshotInferenceRequest {
                model: selected_model.get(),
                num_predict: num_predict.get(),
                prompt: prompt.get(),
                n_batch: n_batch.get(),
                top_k: top_k.get(),
                top_p: top_p.get(),
                repeat_penalty: repeat_penalty.get(),
                temp: temp.get(),
                play_back_tokens: play_back_tokens.get(),
            };
            let resp = api.oneshot_inference(request).await;
            read_inference_stream(
                cx,
                resp,
                authorized_api,
                response_view,
                status_message,
                should_cancel,
            )
            .await;

            is_inference_running.update(|r| *r = false);
        } else {
            status_message.update(|m| {
                *m = Message::Error("failed to connect to API".into());
            });
        }
    });

    let dispatch_inference_action = move || inference_action.dispatch(());

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::PromptGenerate));

        let flex = move || {
            if window_size.get().width < 992 {
                "d-flex flex-column"
            } else {
                "d-flex flex-row"
            }
        };


        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 overflow-auto" >
                 <div class="d-flex">
                     <h1 class="display-5 p-1">{Page::Prompt.title()}</h1>
                 </div>
                 <div class=flex>
                     <div class="col-lg-6 col-sm-12 px-3 pb-3">
                      <div class="card bg-darker p-4 col-12 h-100 overflow-auto">
                        <pre>{response_view}</pre>
                        <Dots is_loading=is_inference_running.read_only() />
                      </div>
                     </div>
                     <div class="col-lg-6 col-sm-12 px-3">
                      <Prompt
                          authorized_api selected_model status_message dispatch_inference_action
                          num_predict prompt n_batch top_k top_p repeat_penalty temp should_cancel
                          is_inference_running play_back_tokens
                      />
                     </div>
                 </div>
           </main>
        }.into_view(cx)
     }}
    }
}

#[component]
fn Prompt<F>(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    status_message: RwSignal<Message>,
    selected_model: RwSignal<String>,
    num_predict: RwSignal<Option<usize>>,
    prompt: RwSignal<String>,
    n_batch: RwSignal<Option<usize>>,
    top_k: RwSignal<Option<usize>>,
    top_p: RwSignal<Option<f32>>,
    repeat_penalty: RwSignal<Option<f32>>,
    temp: RwSignal<Option<f32>>,
    should_cancel: RwSignal<bool>,
    is_inference_running: RwSignal<bool>,
    play_back_tokens: RwSignal<bool>,
    dispatch_inference_action: F,
) -> impl IntoView
where
    F: FnOnce() -> () + Copy + 'static,
{
    let current_list_page = create_rw_signal::<u32>(cx, 1);

    let is_advanced_settings_open = create_rw_signal(cx, true);

    let settings_icon = Signal::derive(cx, move || {
        if is_advanced_settings_open.get() {
            "/icons/minus-circle.svg"
        } else {
            "/icons/plus-circle.svg"
        }
    });
    let models = create_resource(
        cx,
        move || current_list_page.get(),
        move |_current_list_page| async move {
            match authorized_api.get() {
                Some(api) => match api.large_language_models().await {
                    Ok(models) => models,
                    Err(e) => {
                        status_message.update(|msg| *msg = Message::Error(e.to_string()));
                        vec![]
                    }
                },
                None => {
                    status_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    create_effect(cx, move |_| {
        if let Some(models) = models.read(cx) {
            if let Some(first) = models.first() {
                selected_model.update(|m| *m = first.name.clone());
            }
        }
    });

    view! { cx,
        <>
        <div class="card bg-darker col-12 h-100 p-4">
          <div class="card-body">
            <form
              on:submit=|ev|ev.prevent_default()
              class="row text-start"
            >

              <div class="input-group mb-3">
                  <label class="input-group-text">"Prompt"</label>
                  <textarea
                    class = "form-control"
                    rows="10"
                    placeholder = "..."
                    on:keyup = move |ev: ev::KeyboardEvent| {
                        match (&*ev.key(), ev.shift_key()) {
                            ("Enter", false) => {
                              dispatch_inference_action();
                            }
                            _=> {
                              let val = event_target_value(&ev);
                              prompt.update(|v|*v = val);
                            }
                        }
                    }
                  >
                  {prompt}
                  </textarea>
              </div>

              <div class="input-group mb-3">
                <label class="input-group-text">"Model"</label>
                <select
                  class="form-select"
                  id="modelNameSelector"
                  name="model_name"
                  on:change = move |ev| {
                    let val = event_target_value(&ev);
                    selected_model.update(|a| *a = val);
                  }
                >
                { move || {
                  let current = selected_model.get();
                  models.read(cx).unwrap_or_default().into_iter().map(|m| {
                      let name = &m.name;
                      if name == &current {
                      view!{ cx, <option value=name selected>{name}</option> }.into_view(cx)
                      } else {
                      view!{ cx, <option value=name>{name}</option> }.into_view(cx)
                      }
                  }).collect::<Vec<_>>()
                }}
                </select>
              </div>

            <button
                class="btn-btn-airtifex btn-outline rounded mx-auto mb-2"
                on:click=move|_|is_advanced_settings_open.update(|o| *o = !*o)
            >
            <img class="fill-airtifex me-2" src=move || settings_icon.get() />
            "Advanced settings"
            </button>
            {move || {
              if is_advanced_settings_open.get() {
                  view!{ cx,
                  <div>
                      <div class="input-group mb-3">
                          <label class="input-group-text">"Max new tokens"</label>
                          <input
                            class = "form-control"
                            placeholder = "1024"
                            on:keyup = move |ev: ev::KeyboardEvent| {
                              match &*ev.key() {
                                  _=> {
                                    let val = event_target_value(&ev);
                                    num_predict.update(|v|*v = val.parse().ok());
                                  }
                              }
                            }
                          />
                      </div>

                      <div class="input-group mb-3">
                          <label class="input-group-text">"batch size"</label>
                          <input
                            class = "form-control"
                            placeholder = "8"
                            on:keyup = move |ev: ev::KeyboardEvent| {
                              match &*ev.key() {
                                  _=> {
                                    let val = event_target_value(&ev);
                                    n_batch.update(|v|*v = val.parse().ok());
                                  }
                              }
                            }
                          />
                      </div>

                      <div class="row">
                        <div class="input-group mb-3">
                            <label class="input-group-text">"top K"</label>
                            <input
                              class = "form-control"
                              placeholder = "40"
                              on:keyup = move |ev: ev::KeyboardEvent| {
                                match &*ev.key() {
                                    _=> {
                                      let val = event_target_value(&ev);
                                      top_k.update(|v|*v = val.parse().ok());
                                    }
                                }
                              }
                            />
                        </div>
                        <div class="input-group mb-3">
                            <label class="input-group-text">"top P"</label>
                            <input
                              class = "form-control"
                              placeholder = "0.95"
                              on:keyup = move |ev: ev::KeyboardEvent| {
                                match &*ev.key() {
                                    _=> {
                                      let val = event_target_value(&ev);
                                      top_p.update(|v|*v = val.parse().ok());
                                    }
                                }
                              }
                            />
                        </div>
                      </div>

                      <div class="input-group mb-3">
                          <label class="input-group-text">"repeat penalty"</label>
                          <input
                            class = "form-control"
                            placeholder = "1.30"
                            on:keyup = move |ev: ev::KeyboardEvent| {
                              match &*ev.key() {
                                  _=> {
                                    let val = event_target_value(&ev);
                                    repeat_penalty.update(|v|*v = val.parse().ok());
                                  }
                              }
                            }
                          />
                      </div>

                      <div class="input-group mb-3">
                          <label class="input-group-text">"temperature"</label>
                          <input
                            class = "form-control"
                            placeholder = "0.80"
                            on:keyup = move |ev: ev::KeyboardEvent| {
                              match &*ev.key() {
                                  _=> {
                                    let val = event_target_value(&ev);
                                    temp.update(|v|*v = val.parse().ok());
                                  }
                              }
                            }
                          />
                      </div>

                      <div class="form-check form-switch">
                        <input
                          class="form-check-input"
                          type="checkbox" 
                          id="playbackTokensSwitch"
                          prop:checked={move || play_back_tokens.get()}
                          on:input=move |_| play_back_tokens.update(|v| *v = !*v)
                        />
                        <label class="form-check-label" for="playbackTokensSwitch">"Play back tokens"</label>
                      </div>

                  </div>
                  }.into_view(cx)
              } else {
                  view!{ cx,
                  <div>
                  </div>
                  }.into_view(cx)
              }
            }}

            <div class="d-flex flex-row mt-3">
              <button
                  class="btn btn-outline-lighter rounded ms-auto me-1"
                  prop:disabled = move || prompt.get().is_empty()
                  on:click=move |_| dispatch_inference_action()
              >
              <img class="me-2" src="/icons/send.svg" />
              "Submit"
              </button>
              <button
                  class="btn btn-outline-lighter rounded me-auto ms-1"
                  prop:disabled = move || !is_inference_running.get()
                  on:click=move |_| should_cancel.update(|c| *c = true)
              >
              <img class="me-2" src="/icons/x.svg" />
              "Cancel"
              </button>
            </div>
            </form>
            <StatusMessage message=status_message />
          </div>
        </div>
        </>
    }
    .into_view(cx)
}
