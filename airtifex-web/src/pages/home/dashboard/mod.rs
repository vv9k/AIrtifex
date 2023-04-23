mod chat;
mod image;

use chat::*;
use image::*;

use crate::{api::AuthorizedApi, components::status_message::*, pages, web_util};
use airtifex_core::{
    image::ImageModelListEntry,
    llm::{LlmListEntry, UserChatCounters},
};

use leptos::*;

#[component]
pub fn Dashboard(
    cx: Scope,
    authorized_api: RwSignal<Option<AuthorizedApi>>,
    global_message: RwSignal<Message>,
) -> impl IntoView {
    let window_size = web_util::WindowSize::signal(cx).expect("window size");

    let chats = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.chat_list().await {
                    Ok(mut chats) => {
                        chats.reverse();
                        chats
                    }
                    Err(e) => {
                        let e = e.to_string();
                        pages::goto_login_if_expired(cx, &e, authorized_api);
                        global_message.update(|msg| *msg = Message::Error(e));
                        vec![]
                    }
                },
                None => {
                    global_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    let images = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.image_list().await {
                    Ok(mut images) => {
                        images.reverse();
                        images
                    }
                    Err(e) => {
                        let e = e.to_string();
                        pages::goto_login_if_expired(cx, &e, authorized_api);
                        global_message.update(|msg| *msg = Message::Error(e));
                        vec![]
                    }
                },
                None => {
                    global_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    let image_models = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.image_models().await {
                    Ok(models) => models,
                    Err(e) => {
                        global_message.update(|msg| *msg = Message::Error(e.to_string()));
                        vec![]
                    }
                },
                None => {
                    global_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    let llms = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.large_language_models().await {
                    Ok(models) => models,
                    Err(e) => {
                        global_message.update(|msg| *msg = Message::Error(e.to_string()));
                        vec![]
                    }
                },
                None => {
                    global_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );

    let chat_counters = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.user_chat_counters().await {
                    Ok(counters) => counters,
                    Err(e) => {
                        global_message.update(|msg| *msg = Message::Error(e.to_string()));
                        UserChatCounters::default()
                    }
                },
                None => {
                    global_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    UserChatCounters::default()
                }
            }
        },
    );

    let inner_view = move || {
        if window_size.get().width < 992 {
            view! { cx,
                <div class="d-flex flex-row col-12 py-3">
                    <div class="d-flex flex-row col-12">
                        <RecentChats chats />
                    </div>
                </div>
                <div class="d-flex flex-row col-12 pb-3">
                    <div class="d-flex flex-row col-12">
                        <div class="col-6 pe-2">
                        <LlModels models=llms />
                        </div>
                        <div class="col-6 ps-2">
                        <ChatCounters counters=chat_counters />
                        </div>
                    </div>
                </div>
                <div class="d-flex flex-row col-12 pb-3">
                    <div class="d-flex flex-row justify-content-center col-12">
                        <RecentImages images />
                    </div>
                </div>
                <div class="d-flex flex-row col-12 pb-3">
                    <div class="d-flex flex-row col-12">
                        <ImageModels models=image_models />
                    </div>
                </div>
            }
            .into_view(cx)
        } else {
            view! { cx,
                <div class="d-flex flex-row col-12 pb-3">
                    <div class="d-flex flex-row justify-content-center col-6 pe-2">
                        <RecentChats chats />
                    </div>
                    <div class="d-flex flex-row justify-content-center col-6 ps-2">
                        <RecentImages images />
                    </div>
                </div>
                <div class="d-flex flex-row col-12">
                    <div class="d-flex flex-row col-6 pe-2">
                        <div class="col-6 pe-2">
                        <LlModels models=llms />
                        </div>
                        <div class="col-6 ps-2">
                        <ChatCounters counters=chat_counters />
                        </div>
                    </div>
                    <div class="d-flex flex-row col-6 ps-2">
                        <ImageModels models=image_models />
                    </div>
                </div>
            }
            .into_view(cx)
        }
    };

    view! { cx,
        <>
            {inner_view}
        </>
    }
}

#[component]
pub fn ImageModels(cx: Scope, models: Resource<(), Vec<ImageModelListEntry>>) -> impl IntoView {
    view! { cx, { move || {
        if let Some(models) = models.read(cx) {
            if !models.is_empty() {
                return view! { cx,
                <div class="card bg-darker p-3">
                    <h2>"Image models"</h2>
                    <div class="card-body d-flex flex-column">
                    <table style="color: rgba(0,0,0,0) !important;" class="table table-hover table-responsive text-white">
                        <thead>
                        <tr>
                            <th scope="col"></th>
                            // <th class="col-7" scope="col"></th>
                        </tr>
                        </thead>
                        <tbody class="text-center">
                    {
                        let count = 5;
                        models.into_iter().take(count).map(|model| {
                            view!{cx,
                                <tr class="text-white no-border">
                                    <td class="text-airtifex-light">{model.name}</td>
                                    // <td class="text-secondary text-center">{model.description}</td>
                                </tr>
                            }.into_view(cx)
                        }).collect::<Vec<_>>()
                    }
                        </tbody>
                    </table>
                    </div>
                </div>
                }.into_view(cx)
            }
       }
       view!{ cx, <></>}.into_view(cx)
    }}}
    .into_view(cx)
}

#[component]
pub fn LlModels(cx: Scope, models: Resource<(), Vec<LlmListEntry>>) -> impl IntoView {
    view! { cx, { move || {
        if let Some(models) = models.read(cx) {
            if !models.is_empty() {
                return view! { cx,
                <div class="card bg-darker p-3">
                    <h2>"LLMs"</h2>
                    <div class="card-body d-flex flex-column">
                    <table style="color: rgba(0,0,0,0) !important;" class="table table-hover table-responsive text-white">
                        <thead>
                        <tr>
                            <th scope="col"></th>
                            // <th class="col-7" scope="col"></th>
                        </tr>
                        </thead>
                        <tbody class="text-center">
                    {
                        let count = 5;
                        models.into_iter().take(count).map(|model| {
                            view!{cx,
                                <tr class="text-white no-border">
                                    <td class="text-airtifex-light">{model.name}</td>
                                    // <td class="text-secondary text-center">{model.description}</td>
                                </tr>
                            }.into_view(cx)
                        }).collect::<Vec<_>>()
                    }
                        </tbody>
                    </table>
                    </div>
                </div>
                }.into_view(cx)
            }
       }
       view!{ cx, <></>}.into_view(cx)
    }}}
    .into_view(cx)
}
