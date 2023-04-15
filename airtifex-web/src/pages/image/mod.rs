use crate::components::{modal::*, status_message::*};
use crate::{api, Page, PageStack};
use airtifex_core::image::{ImageInspect, TextToImageRequest};

use leptos::*;
use leptos_router::*;

pub mod view;

pub use view::*;

#[component]
pub fn TextToImage(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let current_list_page = create_rw_signal::<u32>(cx, 1);

    let status_message = create_rw_signal(cx, Message::Empty);
    let remove_image_id = create_rw_signal(cx, None::<String>);

    let width = create_rw_signal(cx, None::<i64>);
    let height = create_rw_signal(cx, None::<i64>);
    let prompt = create_rw_signal(cx, String::new());
    let selected_model = create_rw_signal(cx, String::new());
    let n_steps = create_rw_signal(cx, None::<usize>);
    let seed = create_rw_signal(cx, None::<i64>);
    let num_samples = create_rw_signal(cx, None::<i64>);
    let guidance_scale = create_rw_signal(cx, None::<f64>);

    let images = create_resource(
        cx,
        move || current_list_page.get(),
        move |_current_list_page| async move {
            match authorized_api.get() {
                Some(api) => match api.images().await {
                    Ok(images) => images,
                    Err(e) => {
                        let e = e.to_string();
                        crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                        status_message.update(|msg| *msg = Message::Error(e));
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

    let remove_image_action = create_action(cx, move |_| async move {
        if let Some(api) = authorized_api.get() {
            if let Some(id) = remove_image_id.get() {
                if let Err(e) = api.image_delete(&id).await {
                    let e = e.to_string();
                    crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                    status_message.update(|m| {
                        *m = Message::Error(format!("failed to remove image - {e}"));
                    });
                } else {
                    status_message.update(|m| {
                        *m = Message::Success(format!("successfully removed image {id}"));
                    });
                    current_list_page.update(|p| *p = *p);
                }
            }
        } else {
            status_message.update(|m| {
                *m = Message::Error("failed to connect to API".into());
            });
        }
    });

    let new_image_action = create_action(cx, move |_| async move {
        if let Some(api) = authorized_api.get() {
            let request = TextToImageRequest {
                prompt: prompt.get(),
                model: selected_model.get(),
                width: width.get(),
                height: height.get(),
                n_steps: n_steps.get(),
                seed: seed.get(),
                num_samples: num_samples.get(),
                guidance_scale: guidance_scale.get(),
            };
            match api.text_to_image(request).await {
                Ok(response) => {
                    status_message.update(|m| {
                        *m = Message::Success(format!(
                            "successfuly registered image {}",
                            response.image_id
                        ));
                    });
                    current_list_page.update(|p| *p = *p);
                }
                Err(e) => {
                    status_message.update(|m| {
                        *m = Message::Error(format!("failed to start a new chat - {e}"));
                    });
                }
            }
        } else {
            status_message.update(|m| {
                *m = Message::Error("failed to connect to API".into());
            });
        }
    });

    let dispatch_new_image_action = move || new_image_action.dispatch(());
    let dispatch_remove_image_action = move || remove_image_action.dispatch(());

    let remove_confirm_modal = move || {
        view! { cx,
          <RemoveModal
            modal_id="removeImageModal"
            target="chat"
            entry=remove_image_id.read_only()
            remove_action_fn=dispatch_remove_image_action
          />
        }
        .into_view(cx)
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::TextToImage));

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3 overflow-auto" >
                 <div class="d-flex pb-3">
                     <h1 class="display-5 p-1">{Page::TextToImage.title()}</h1>
                 </div>
                 <TextToImageForm
                     authorized_api status_message prompt width height n_steps seed num_samples
                     selected_model dispatch_new_image_action guidance_scale
                 />
                 <div class="card bg-darker m-3">
                    <StatusMessage message=status_message />
                    <ImageListEntries images remove_image_id />
                 </div>
           </main>
           {remove_confirm_modal}
        }.into_view(cx)
     }}
    }
}

#[component]
fn TextToImageForm<F>(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    status_message: RwSignal<Message>,
    prompt: RwSignal<String>,
    width: RwSignal<Option<i64>>,
    height: RwSignal<Option<i64>>,
    n_steps: RwSignal<Option<usize>>,
    seed: RwSignal<Option<i64>>,
    num_samples: RwSignal<Option<i64>>,
    guidance_scale: RwSignal<Option<f64>>,
    selected_model: RwSignal<String>,
    dispatch_new_image_action: F,
) -> impl IntoView
where
    F: FnOnce() -> () + Copy + 'static,
{
    let current_list_page = create_rw_signal(cx, 1);
    let is_advanced_settings_open = create_rw_signal(cx, false);

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
                Some(api) => match api.image_models().await {
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
        <div class="card bg-darker m-3">
                  <div class="card-body w-50 mx-auto pb-3 pt-5">
                    <form
                      on:submit=|ev|ev.prevent_default()
                      class="row text-start"
                    >

                      <div class="input-group mb-3">
                         <label class="input-group-text">"Prompt"</label>
                         <input
                           class = "form-control"
                           placeholder = "..."
                           on:keyup = move |ev: ev::KeyboardEvent| {
                             match &*ev.key() {
                                 "Enter" => {
                                    dispatch_new_image_action();
                                 }
                                 _=> {
                                    let val = event_target_value(&ev);
                                    prompt.update(|v|*v = val);
                                 }
                             }
                           }
                         />
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
                                 <label class="input-group-text">"Width"</label>
                                 <input
                                   class = "form-control"
                                   placeholder = "256"
                                   on:keyup = move |ev: ev::KeyboardEvent| {
                                     match &*ev.key() {
                                         "Enter" => {
                                            dispatch_new_image_action();
                                         }
                                         _=> {
                                            let val = event_target_value(&ev);
                                            width.update(|v|*v = val.parse().ok());
                                         }
                                     }
                                   }
                                 />
                              </div>

                              <div class="input-group mb-3">
                                 <label class="input-group-text">"Height"</label>
                                 <input
                                   class = "form-control"
                                   placeholder = "256"
                                   on:keyup = move |ev: ev::KeyboardEvent| {
                                     match &*ev.key() {
                                         "Enter" => {
                                            dispatch_new_image_action();
                                         }
                                         _=> {
                                            let val = event_target_value(&ev);
                                            height.update(|v|*v = val.parse().ok());
                                         }
                                     }
                                   }
                                 />
                              </div>

                              <div class="row">
                                <div class="input-group mb-3">
                                   <label class="input-group-text">"N Steps"</label>
                                   <input
                                     class = "form-control"
                                     placeholder = "15"
                                     on:keyup = move |ev: ev::KeyboardEvent| {
                                       match &*ev.key() {
                                           "Enter" => {
                                              dispatch_new_image_action();
                                           }
                                           _=> {
                                              let val = event_target_value(&ev);
                                              n_steps.update(|v|*v = val.parse().ok());
                                           }
                                       }
                                     }
                                   />
                                </div>
                                <div class="input-group mb-3">
                                   <label class="input-group-text">"Seed"</label>
                                   <input
                                     class = "form-control"
                                     placeholder = "1337"
                                     on:keyup = move |ev: ev::KeyboardEvent| {
                                       match &*ev.key() {
                                           "Enter" => {
                                              dispatch_new_image_action();
                                           }
                                           _=> {
                                              let val = event_target_value(&ev);
                                              seed.update(|v|*v = val.parse().ok());
                                           }
                                       }
                                     }
                                   />
                                </div>
                              </div>

                              <div class="input-group mb-3">
                                 <label class="input-group-text">"N samples"</label>
                                 <input
                                   class = "form-control"
                                   placeholder = "1"
                                   on:keyup = move |ev: ev::KeyboardEvent| {
                                     match &*ev.key() {
                                         "Enter" => {
                                            dispatch_new_image_action();
                                         }
                                         _=> {
                                            let val = event_target_value(&ev);
                                            num_samples.update(|v|*v = val.parse().ok());
                                         }
                                     }
                                   }
                                 />
                              </div>

                              <div class="input-group mb-3">
                                 <label class="input-group-text">"Guidance Scale"</label>
                                 <input
                                   class = "form-control"
                                   placeholder = "7.5"
                                   on:keyup = move |ev: ev::KeyboardEvent| {
                                     match &*ev.key() {
                                         "Enter" => {
                                            dispatch_new_image_action();
                                         }
                                         _=> {
                                            let val = event_target_value(&ev);
                                            guidance_scale.update(|v|*v = val.parse().ok());
                                         }
                                     }
                                   }
                                 />
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

                      <button
                         class="btn btn-outline-lighter rounded mt-3 w-25 mx-auto"
                         on:click=move |_| dispatch_new_image_action()
                      >
                      <img class="me-2" src="/icons/send.svg" />
                      "New image"
                      </button>
                    </form>
                  </div>
                </div>
        </>
    }
    .into_view(cx)
}

#[component]
fn ImageListEntries(
    cx: Scope,
    images: Resource<u32, Vec<ImageInspect>>,
    remove_image_id: RwSignal<Option<String>>,
) -> impl IntoView {
    view! { cx, { move || {
        if let Some(images) = images.read(cx) {
            if !images.is_empty() {
                return view! { cx,
                <div class="card-body d-flex flex-column px-5 pb-5">
                  <table class="table table-hover table-striped table-responsive text-white">
                    <thead>
                    <tr>
                      <th class="col-3" scope="col">"Prompt"</th>
                      <th scope="col">"Model"</th>
                      <th scope="col">"Width"</th>
                      <th scope="col">"Height"</th>
                      <th scope="col">"Seed"</th>
                      <th scope="col">"N Steps"</th>
                      <th scope="col">"N Samples"</th>
                      <th scope="col">"Finished"</th>
                      <th scope="col"></th>
                    </tr>
                    </thead>
                    <tbody>
                   {
                      images.into_iter().map(|image| {
                          view!{cx, <ImageListEntry image remove_image_id />}.into_view(cx)
                      }).collect::<Vec<_>>()
                   }
                    </tbody>
                  </table>
                </div>
                }.into_view(cx)
            }
       }
       view!{ cx, <></>}.into_view(cx)
    }}}
    .into_view(cx)
}

#[component]
fn ImageListEntry(
    cx: Scope,
    image: ImageInspect,
    remove_image_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let view_href = format!("/image/tti/{}", image.id);
    let is_finished = if !image.processing {
        view! { cx, <span class="text-airtifex-green">"✓"</span>}
    } else {
        view! { cx, <span class="text-airtifex-yellow">"✗"</span>}
    };
    view! {cx, <tr
                class="text-white border-lighter"
              >
                  <td>{image.prompt}</td>
                  <td>{image.model}</td>
                  <td>{image.width}</td>
                  <td>{image.height}</td>
                  <td>{image.seed}</td>
                  <td>{image.n_steps}</td>
                  <td>{image.num_samples}</td>
                  <td>{is_finished}</td>
                  <td>
                      <div class="btn-group" role="chat toolbar" aria-label="chat toolbar">
                          <button
                            class="btn btn-outline-lighter"
                            data-bs-toggle="modal"
                            data-bs-target="#removeImageModal"
                            on:focus=move |_| {
                                remove_image_id.update(|c| *c = Some(image.id.clone()));
                            }
                          >
                              <img src="/icons/minus-circle.svg" class="me-2" />
                              "Remove"
                          </button>
                          <button
                            class="btn btn-outline-lighter"
                            on:click=move |_| {
                                let navigate = use_navigate(cx);
                                navigate(&view_href, Default::default()).expect("image page");
                            }
                          >
                              <img src="/icons/edit.svg" class="me-2" />
                              "Open"
                          </button>
                      </div>
                  </td>
              </tr>
    }
    .into_view(cx)
}
