use crate::{
    api,
    components::{modal::*, status_message::*},
    pages, web_util, Page, PageStack,
};
use airtifex_core::image::{ImageGenerateRequest, ImageInspect, InputImage};

use leptos::*;

pub mod view;

pub use view::*;

#[component]
pub fn GenerateImage(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let current_list_page = create_rw_signal::<u32>(cx, 1);

    let status_message = create_rw_signal(cx, Message::Empty);
    let remove_image_id = create_rw_signal(cx, None::<String>);

    let input_image = create_rw_signal(cx, None::<web_sys::File>);
    let strength = create_rw_signal(cx, 0.7);
    let mask = create_rw_signal(cx, None::<web_sys::File>);

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
                Some(api) => match api.image_list().await {
                    Ok(images) => images,
                    Err(e) => {
                        let e = e.to_string();
                        pages::goto_login_if_expired(cx, &e, authorized_api);
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
                    pages::goto_login_if_expired(cx, &e, authorized_api);
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
            let data = if let Some(f) = input_image.get() {
                match web_util::read_file(f).await {
                    Ok(data) => Some(data),
                    Err(e) => {
                        status_message.update(|m| {
                            *m = Message::Error(format!(
                                "failed to read input image - {}",
                                e.as_string().unwrap_or_default()
                            ));
                        });
                        return;
                    }
                }
            } else {
                None
            };
            let mask = if let Some(f) = mask.get() {
                match web_util::read_file(f).await {
                    Ok(data) => Some(data),
                    Err(e) => {
                        status_message.update(|m| {
                            *m = Message::Error(format!(
                                "failed to read mask image - {}",
                                e.as_string().unwrap_or_default()
                            ));
                        });
                        return;
                    }
                }
            } else {
                None
            };
            let input_image = data.map(|d| InputImage {
                data: d,
                mask,
                strength: Some(strength.get()),
            });
            let request = ImageGenerateRequest {
                prompt: prompt.get(),
                input_image,
                model: selected_model.get(),
                width: width.get(),
                height: height.get(),
                n_steps: n_steps.get(),
                seed: seed.get(),
                num_samples: num_samples.get(),
                guidance_scale: guidance_scale.get(),
            };
            match api.image_generate(request).await {
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
                        *m = Message::Error(format!("failed to register image - {e}"));
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
        page_stack.update(|p| p.push(Page::GenerateImage));

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3 overflow-auto" >
                 <div class="d-flex pb-3">
                     <h1 class="display-5 p-1">{Page::GenerateImage.title()}</h1>
                 </div>
                 <GenerateImageForm
                     authorized_api status_message prompt width height n_steps seed num_samples
                     selected_model dispatch_new_image_action guidance_scale input_image mask
                     strength
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
fn GenerateImageForm<F>(
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
    input_image: RwSignal<Option<web_sys::File>>,
    mask: RwSignal<Option<web_sys::File>>,
    strength: RwSignal<f64>,
    dispatch_new_image_action: F,
) -> impl IntoView
where
    F: FnOnce() -> () + Copy + 'static,
{
    let current_list_page = create_rw_signal(cx, 1);
    let is_advanced_settings_open = create_rw_signal(cx, false);
    let is_input_image_visible = create_rw_signal(cx, false);
    let is_mask_visible = create_rw_signal(cx, false);

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
                is_input_image_visible
                    .update(|v| *v = first.features.inpaint || first.features.image_to_image);
                is_mask_visible.update(|v| *v = first.features.inpaint);
            }
        }
    });

    view! { cx,
        <>
        <div class="card bg-darker m-3">
                  <div class="card-body col-6 col-sm-9 mx-auto pb-3 pt-5">
                    <form
                      on:submit=|ev|ev.prevent_default()
                      class="row text-start"
                    >

                      <div class="input-group mb-3">
                         <label class="input-group-text">"Prompt"</label>
                         <textarea
                           class = "form-control"
                           required
                           rows="2"
                           value=move || prompt.get()
                           placeholder = "..."
                           on:keyup = move |ev: ev::KeyboardEvent| {
                                match (&*ev.key(), ev.shift_key()) {
                                    ("Enter", false) => {
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
                      {move || if is_input_image_visible.get() {
                        view!{ cx,
                        <div class="input-group mb-3">
                            <label class="input-group-text">"Input Image (optional)"</label>
                            <input
                            type="file"
                            accept="image/png, image/jpeg"
                            class="form-control"
                            on:change = move |ev| {
                                input_image.update(|v| *v = web_util::extract_file_from_html_input(ev));
                            }
                            />
                        </div>
                        }.into_view(cx)
                      } else {
                        view!{ cx, <></> }.into_view(cx)
                      }}
                      {move || if input_image.get().is_some() {
                        view!{ cx,
                        <div class="input-group mb-3">
                            <label class="form-label">"Replacement strength: "{strength}</label>
                            <input
                            type="range"
                            class = "form-range"
                            min = "0"
                            max = "1"
                            step = "0.01"
                            value = {move || strength.get()}
                            on:change = move |ev| {
                                let val = event_target_value(&ev);
                                strength.update(|v|*v = val.parse().ok().unwrap_or(0.7));
                            }
                            />
                        </div>
                        }.into_view(cx)
                      } else {
                        view!{ cx, <></> }.into_view(cx)
                      }}

                      {move || if is_mask_visible.get() {
                        view!{ cx,
                        <div class="input-group mb-3">
                          <label class="input-group-text">"Mask (optional)"</label>
                          <input
                            type="file"
                            accept="image/png, image/jpeg"
                            class="form-control"
                            on:change = move |ev| {
                              mask.update(|v| *v = web_util::extract_file_from_html_input(ev));
                            }
                          />
                        </div>
                        }.into_view(cx)
                      } else {
                        view!{ cx, <></> }.into_view(cx)
                      }}

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
                         class="btn btn-outline-lighter rounded mt-3 col-lg-3 col-sm-6 mx-auto"
                         on:click=move |_| dispatch_new_image_action()
                         prop:disabled=move || prompt.get().is_empty()
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
                      <th scope="col">""</th>
                      <th class="col-3" scope="col">"Prompt"</th>
                      <th class="text-center" scope="col">"Model"</th>
                      <th class="text-center" scope="col">"Width"</th>
                      <th class="text-center" scope="col">"Height"</th>
                      <th class="text-center" scope="col">"Seed"</th>
                      <th class="text-center" scope="col">"N Steps"</th>
                      <th class="text-center" scope="col">"N Samples"</th>
                      <th class="text-center" scope="col">"Finished"</th>
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
    let view_href = format!("{}/{}", Page::GenerateImage.raw_path(), image.id);
    let view_href2 = view_href.clone();
    let is_finished = if !image.processing {
        view! { cx, <span class="text-airtifex-green">"✓"</span>}
    } else {
        view! { cx, <span class="text-airtifex-yellow">"✗"</span>}
    };
    view! {cx, <tr
                class="text-white no-border align-middle"
              >
                  <td class="fitwidth">
                  { move || {
                    if let Some(thumbnail) = &image.thumbnail {
                        let image = web_util::encode_image_base64(thumbnail);
                        view! { cx, <img src=image />}.into_view(cx)
                    } else {
                        view! { cx, <></> }.into_view(cx)
                    }
                  }}
                  </td>
                  <td
                    style="cursor: pointer;"
                    on:click=move |_| {
                        pages::goto(cx, &view_href2).expect("image page");
                    }
                  >
                    {image.prompt}
                  </td>
                  <td align="center" class="text-airtifex-light">{image.model}</td>
                  <td align="center">{image.width}</td>
                  <td align="center">{image.height}</td>
                  <td align="center">{image.seed}</td>
                  <td align="center">{image.n_steps}</td>
                  <td align="center">{image.num_samples}</td>
                  <td align="center">{is_finished}</td>
                  <td align="right">
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
                                pages::goto(cx, &view_href).expect("image page");
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
