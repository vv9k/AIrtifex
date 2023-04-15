use crate::components::{status_message::*, titled_child_page::*};
use crate::inference::read_inference_stream;
use crate::{api, wasm_sleep, Page, PageStack};
use airtifex_core::llm::ChatResponseRequest;

use leptos::*;
use leptos_router::*;

#[derive(Clone, Copy, Debug)]
pub enum Entry {
    User,
    Chat,
    None,
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ChatParams {
    chat_id: Option<String>,
}

#[component]
pub fn ChatView(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let params = use_params::<ChatParams>(cx);

    let current_list_page = create_rw_signal::<u32>(cx, 1);
    let dummy_chat_signal = create_rw_signal::<u32>(cx, 1);
    let prompt = create_rw_signal(cx, String::new());
    let responses = create_rw_signal(cx, vec![]);
    let last_response = create_rw_signal(cx, (Entry::None, String::new()));
    let infered_response = create_rw_signal(cx, String::new());
    let status_message = create_rw_signal(cx, Message::Empty);

    let is_inference_running = create_rw_signal(cx, false);
    let should_cancel = create_rw_signal(cx, false);

    let is_details_open = create_rw_signal(cx, false);

    let chat_id = Signal::derive(cx, move || params.get().ok().and_then(|p| p.chat_id));

    let chat = create_resource(
        cx,
        move || dummy_chat_signal.get(),
        move |_| async move {
            match (authorized_api.get(), chat_id.get()) {
                (Some(api), Some(id)) => match api.chat(&id).await {
                    Ok(chat) => Some(chat),
                    Err(e) => {
                        let e = e.to_string();
                        crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                        status_message.update(|msg| *msg = Message::Error(e));
                        None
                    }
                },
                _ => {
                    status_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    None
                }
            }
        },
    );

    let title = Signal::derive(cx, move || {
        chat.read(cx)
            .and_then(|e| e.map(|e| e.title))
            .unwrap_or("Chat".into())
    });
    let model = Signal::derive(cx, move || {
        chat.read(cx)
            .and_then(|e| e.map(|e| e.model))
            .unwrap_or("".into())
    });

    let history = create_resource(
        cx,
        move || current_list_page.get(),
        move |_current_list_page| async move {
            match (authorized_api.get(), chat_id.get()) {
                (Some(api), Some(id)) => match api.chat_history(&id).await {
                    Ok(chats) => {
                        log::info!("got chats {chats:?}");
                        chats
                    }
                    Err(e) => {
                        let e = e.to_string();
                        crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                        status_message.update(|msg| *msg = Message::Error(e));
                        vec![]
                    }
                },
                _ => {
                    status_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    create_effect(cx, move |_| {
        if let Some(history) = history.read(cx) {
            responses.update(|rsp| {
                *rsp = history
                    .into_iter()
                    .map(|entry| {
                        let ty = match entry.entry_type {
                            airtifex_core::llm::ChatEntryType::Bot => Entry::Chat,
                            airtifex_core::llm::ChatEntryType::User => Entry::User,
                        };
                        (ty, entry.content)
                    })
                    .collect();
            });
        }
    });

    create_effect(cx, move |_| {
        if !infered_response.get().is_empty() {
            last_response.update(|r| *r = (Entry::Chat, infered_response.get()));
        }
    });

    let prompt_submit_action = create_action(cx, move |p: &String| {
        let p = p.clone();
        let request = ChatResponseRequest {
            prompt: p,
            ..Default::default()
        };
        async move {
            let id = if let Some(id) = chat_id.get() {
                id
            } else {
                status_message.update(|m| {
                    *m = Message::Error("chat ID missing".into());
                });
                return;
            };
            if let Some(api) = authorized_api.get() {
                if is_inference_running.get() && !should_cancel.get() {
                    should_cancel.update(|c| *c = true);
                    // wait for any other job to cancel
                    let _ = wasm_sleep(100).await;
                }
                is_inference_running.update(|r| *r = true);
                status_message.update(|s| *s = Message::Success("Generating...".into()));
                responses.update(|rsp| {
                    rsp.push((Entry::User, request.prompt.clone()));
                });
                let resp = api.get_chat_response(request, &id).await;
                read_inference_stream(
                    cx,
                    resp,
                    authorized_api,
                    infered_response,
                    status_message,
                    should_cancel,
                )
                .await;
                responses.update(|rsp| rsp.push(last_response.get()));
                last_response.update(|(e, rsp)| {
                    *e = Entry::None;
                    rsp.clear()
                });

                is_inference_running.update(|r| *r = false);
            } else {
                status_message.update(|m| {
                    *m = Message::Error("failed to connect to API".into());
                });
            }
        }
    });

    let dispatch_prompt_submit = move || {
        prompt_submit_action.dispatch(prompt.get());
        prompt.update(|v| *v = "".into())
    };

    let settings = move || {
        if let Some(Some(chat)) = chat.read(cx) {
            let icon = if is_details_open.get() {
                "/icons/minus-circle.svg"
            } else {
                "/icons/plus-circle.svg"
            };
            view! { cx,
             <button
                class="btn-btn-airtifex btn-outline rounded me-auto ms-2 mb-2"
                on:click=move|_|is_details_open.update(|o| *o = !*o)
             >
              <img class="me-2" src=icon />
              "Details"
             </button>
             { if is_details_open.get() {
                 view!{ cx,
                 <table class="table table-hover table-striped table-responsive text-white">
                   <thead>
                    <tr>
                     <th scope="col" class="border-0 font-monospace">"Num predict: "<span class="text-airtifex-yellow">{chat.settings.num_predict}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Batch size: "<span class="text-airtifex-yellow">{chat.settings.n_batch}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Top K: "<span class="text-airtifex-yellow">{chat.settings.top_k}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Top P: "<span class="text-airtifex-yellow">{chat.settings.top_p}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Repeat penalty: "<span class="text-airtifex-yellow">{chat.settings.repeat_penalty}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Temperature: "<span class="text-airtifex-yellow">{chat.settings.temp}</span></th>
                   </tr>
                   </thead>
                 </table>
                 }.into_view(cx)
             } else {
                view! { cx, <></> }.into_view(cx)
             }}
            }
            .into_view(cx)
        } else {
            view! { cx, <></> }.into_view(cx)
        }
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::ChatView));

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3" >
             <TitledChildPage title={title} parent_href=Page::Chat.path()></TitledChildPage>
             <div class="text-center w-100">
                 <p class="text-airtifex-light font-monospace">{model}</p>
             </div>
             {settings}
             <div class="d-flex justify-content-between flex-column h-100 w-100 overflow-auto">
                 <div class="px-5 py-2">
                   <div class="w-100 h-100">
                       { move || {
                           responses.get().iter().chain([last_response.get()].iter()).map(|(entry, rsp)| {
                               let (class, prefix) = match entry {
                                   Entry::User => ("fs-5","User: "),
                                   Entry::Chat => ("text-airtifex-light fs-5", "Chat: "),
                                   Entry::None => ("fs-5", ""),
                               };
                               view!{cx, <p><strong class=class>{prefix}</strong><pre class="fs-6 ms-3">{rsp}</pre></p>
                               }}.into_view(cx)).collect::<Vec<_>>()
                       }}
                       <p style="height: 12rem"></p>
                   </div>
                 </div>
                 <div style="transform: TranslateX(-31.67%);" class="w-50 mx-auto start-50 bottom-0 position-fixed pb-3">
                       <form
                         on:submit=|ev|ev.prevent_default()
                         class="row text-start"
                       >
                           <div class="input-group mb-3">
                               <textarea
                                 class = "form-control"
                                 required
                                 rows="2"
                                 value=move || prompt.get()
                                 placeholder = "Enter your prompt..."
                                 on:keyup = move |ev: ev::KeyboardEvent| {
                                   match (&*ev.key(), ev.shift_key()) {
                                       ("Enter", false) => {
                                          dispatch_prompt_submit();
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
                        <div class="d-flex flex-row mt-3">
                        <button
                            class="btn btn-outline-lighter rounded ms-auto me-1"
                            prop:disabled = move || prompt.get().is_empty()
                            on:click=move |_| dispatch_prompt_submit()
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
                       <div class="mt-4">
                       <StatusMessage message=status_message></StatusMessage>
                       </div>
                 </div>
             </div>
           </main>
        }.into_view(cx)
     }}
    }
}