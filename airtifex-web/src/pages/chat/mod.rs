use crate::components::{modal::*, status_message::*};
use crate::{api, Page, PageStack};
use airtifex_core::llm::{ChatListEntry, ChatStartRequest, InferenceSettings};

use leptos::*;
use leptos_router::*;

pub mod view;
pub use view::*;

#[component]
pub fn Chat(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let current_list_page = create_rw_signal::<u32>(cx, 1);

    let status_message = create_rw_signal(cx, Message::Empty);
    let remove_chat_id = create_rw_signal(cx, None::<String>);
    let remove_chat_title = create_rw_signal(cx, None::<String>);

    let chat_title = create_rw_signal(cx, String::new());
    let selected_model = create_rw_signal(cx, String::new());

    let num_predict = create_rw_signal(cx, None::<usize>);
    let n_batch = create_rw_signal(cx, None::<usize>);
    let top_k = create_rw_signal(cx, None::<usize>);
    let top_p = create_rw_signal(cx, None::<f32>);
    let repeat_penalty = create_rw_signal(cx, None::<f32>);
    let temp = create_rw_signal(cx, None::<f32>);

    let chats = create_resource(
        cx,
        move || current_list_page.get(),
        move |_current_list_page| async move {
            match authorized_api.get() {
                Some(api) => match api.chat_list().await {
                    Ok(chats) => chats,
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

    let remove_chat_action = create_action(cx, move |_| async move {
        if let Some(api) = authorized_api.get() {
            if let (Some(title), Some(id)) = (remove_chat_title.get(), remove_chat_id.get()) {
                if let Err(e) = api.chat_remove(&id).await {
                    let e = e.to_string();
                    crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                    status_message.update(|m| {
                        *m = Message::Error(format!("failed to remove chat - {e}"));
                    });
                } else {
                    status_message.update(|m| {
                        *m = Message::Success(format!("successfully removed chat \"{title}\""));
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

    let new_chat_action = create_action(cx, move |_| async move {
        if let Some(api) = authorized_api.get() {
            let title = chat_title.get();
            let title = if title.is_empty() { None } else { Some(title) };
            let request = ChatStartRequest {
                title,
                model: Some(selected_model.get()),
                settings: InferenceSettings {
                    num_predict: num_predict.get(),
                    system_prompt: None,
                    n_batch: n_batch.get(),
                    top_k: top_k.get(),
                    top_p: top_p.get(),
                    repeat_penalty: repeat_penalty.get(),
                    temp: temp.get(),
                },
            };
            match api.chat_start_new(request).await {
                Ok(response) => {
                    let navigate = use_navigate(cx);
                    navigate(&format!("/chat/{}", response.chat_id), Default::default())
                        .expect("chat page");
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

    let dispatch_new_chat_action = move || new_chat_action.dispatch(());
    let dispatch_remove_chat_action = move || remove_chat_action.dispatch(());

    let remove_confirm_modal = move || {
        view! { cx,
          <RemoveModal
            modal_id="removeChatModal"
            target="chat"
            entry=remove_chat_title.read_only()
            remove_action_fn=dispatch_remove_chat_action
          />
        }
        .into_view(cx)
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::Chat));

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3 overflow-auto" >
                 <div class="d-flex pb-3">
                     <h1 class="display-5 p-1">{Page::Chat.title()}</h1>
                 </div>
                 <NewChatForm
                     authorized_api selected_model status_message chat_title dispatch_new_chat_action
                     num_predict n_batch top_k top_p repeat_penalty temp
                 />
                 <div class="card bg-darker m-3">
                    <StatusMessage message=status_message />
                    <ChatListEntries chats remove_chat_id=remove_chat_id remove_chat_title=remove_chat_title />
                 </div>
           </main>
           {remove_confirm_modal}
        }.into_view(cx)
     }}
    }
}

#[component]
fn NewChatForm<F>(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    status_message: RwSignal<Message>,
    selected_model: RwSignal<String>,
    chat_title: RwSignal<String>,
    num_predict: RwSignal<Option<usize>>,
    n_batch: RwSignal<Option<usize>>,
    top_k: RwSignal<Option<usize>>,
    top_p: RwSignal<Option<f32>>,
    repeat_penalty: RwSignal<Option<f32>>,
    temp: RwSignal<Option<f32>>,
    dispatch_new_chat_action: F,
) -> impl IntoView
where
    F: FnOnce() -> () + Copy + 'static,
{
    let current_list_page = create_rw_signal::<u32>(cx, 1);

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
        <div class="card bg-darker m-3">
                  <div class="card-body col-6 col-sm-9 mx-auto pb-3 pt-5">
                    <form
                      on:submit=|ev|ev.prevent_default()
                      class="row text-start"
                    >

                      <div class="input-group mb-3">
                         <label class="input-group-text">"Title"</label>
                         <input
                           class = "form-control"
                           placeholder = "..."
                           on:keyup = move |ev: ev::KeyboardEvent| {
                             match &*ev.key() {
                                 "Enter" => {
                                    dispatch_new_chat_action();
                                 }
                                 _=> {
                                    let val = event_target_value(&ev);
                                    chat_title.update(|v|*v = val);
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
                                 <label class="input-group-text">"Max new tokens"</label>
                                 <input
                                   class = "form-control"
                                   placeholder = "1024"
                                   on:keyup = move |ev: ev::KeyboardEvent| {
                                     match &*ev.key() {
                                         "Enter" => {
                                            dispatch_new_chat_action();
                                         }
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
                                         "Enter" => {
                                            dispatch_new_chat_action();
                                         }
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
                                           "Enter" => {
                                              dispatch_new_chat_action();
                                           }
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
                                           "Enter" => {
                                              dispatch_new_chat_action();
                                           }
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
                                         "Enter" => {
                                            dispatch_new_chat_action();
                                         }
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
                                         "Enter" => {
                                            dispatch_new_chat_action();
                                         }
                                         _=> {
                                            let val = event_target_value(&ev);
                                            temp.update(|v|*v = val.parse().ok());
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
                         on:click=move |_| dispatch_new_chat_action()
                      >
                      <img class="me-2" src="/icons/message-circle.svg" />
                      "New chat"
                      </button>
                    </form>
                  </div>
                </div>
        </>
    }
    .into_view(cx)
}

#[component]
fn ChatListEntries(
    cx: Scope,
    chats: Resource<u32, Vec<ChatListEntry>>,
    remove_chat_id: RwSignal<Option<String>>,
    remove_chat_title: RwSignal<Option<String>>,
) -> impl IntoView {
    view! { cx, { move || {
        if let Some(chats) = chats.read(cx) {
            if !chats.is_empty() {
                return view! { cx,
                <div class="card-body d-flex flex-column px-5 pb-5">
                  <table class="table table-hover table-striped table-responsive text-white">
                    <thead>
                    <tr>
                      <th scope="col">"Previous conversations"</th>
                      <th scope="col">"Model"</th>
                      <th scope="col">"Start date"</th>
                      <th scope="col"></th>
                    </tr>
                    </thead>
                    <tbody>
                   {
                      chats.into_iter().map(|chat| {
                          view!{cx, <ChatListEntry chat remove_chat_id remove_chat_title />}.into_view(cx)
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
fn ChatListEntry(
    cx: Scope,
    chat: ChatListEntry,
    remove_chat_id: RwSignal<Option<String>>,
    remove_chat_title: RwSignal<Option<String>>,
) -> impl IntoView {
    let edit_href = format!("/chat/{}", chat.id);
    view! {cx, <tr
                class="text-white border-lighter"
              >
                  <td>
                    {chat.title.clone()}
                  </td>
                  <td>{chat.model}</td>
                  <td>{chat.start_date.format("%a, %d %b %Y %H:%M:%S").to_string()}</td>
                  <td>
                      <div class="btn-group" role="chat toolbar" aria-label="chat toolbar">
                          <button
                            class="btn btn-outline-lighter"
                            data-bs-toggle="modal"
                            data-bs-target="#removeChatModal"
                            on:focus=move |_| {
                                remove_chat_id.update(|c| *c = Some(chat.id.clone()));
                                remove_chat_title.update(|c| *c = Some(chat.title.clone()))
                            }
                          >
                              <img src="/icons/minus-circle.svg" class="me-2" />
                              "Remove"
                          </button>
                          <button
                            class="btn btn-outline-lighter"
                            on:click=move |_| {
                                let navigate = use_navigate(cx);
                                navigate(&edit_href, Default::default()).expect("chat page");
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
