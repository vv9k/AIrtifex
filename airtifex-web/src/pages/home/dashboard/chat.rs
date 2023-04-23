use crate::Page;

use airtifex_core::{
    llm::{ChatListEntry, UserChatCounters},
};
use leptos::*;

#[component]
pub fn RecentChats(cx: Scope, chats: Resource<(), Vec<ChatListEntry>>) -> impl IntoView {
    view! { cx, { move || {
        if let Some(chats) = chats.read(cx) {
            if !chats.is_empty() {
                return view! { cx,
                <div class="card bg-darker p-3 col-12">
                    <h2>"Recent Chats"</h2>
                    <div class="card-body d-flex flex-column">
                    <table style="color: rgba(0,0,0,0) !important;" class="table table-hover table-responsive text-white">
                        <thead>
                        <tr>
                        <th class="col-8" scope="col">""</th>
                        <th scope="col"></th>
                        </tr>
                        </thead>
                        <tbody class="text-start">
                    {
                        const DISPLAY_COUNT: usize = 5;
                        let count = chats.len();
                        let mut views = chats.into_iter().take(DISPLAY_COUNT).map(|chat| {
                            view!{cx, <RecentChatEntry chat />}.into_view(cx)
                        }).collect::<Vec<_>>();

                        if count > DISPLAY_COUNT {
                            views.push(view!{ cx, 
                                <tr 
                                  class="text-white no-border"
                                  style="cursor: pointer;"
                                  on:click={move |_| {
                                      crate::pages::goto(cx, Page::Chat).expect("chat page");
                                  }}
                                >
                                    <td 
                                      class="text-secondary text-center" 
                                      colspan="2" 
                                      scope="col"
                                    >
                                        <img src="/icons/list.svg" class="me-2" />
                                        "view more..."
                                    </td>
                                </tr>
                            }.into_view(cx));
                        }
                        views
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
fn RecentChatEntry(cx: Scope, chat: ChatListEntry) -> impl IntoView {
    let view_href = format!("{}/{}", Page::Chat.path(), chat.id);
    view! {cx, <tr
                class="text-white no-border"
                style="cursor: pointer;"
                on:click={move |_| {
                    crate::pages::goto(cx, &view_href).expect("chat page");
                }}
              >
                  <td>
                    {chat.title.clone()}
                  </td>
                  <td class="text-airtifex-light text-center">{chat.model}</td>
              </tr>
    }
    .into_view(cx)
}

#[component]
pub fn ChatCounters(cx: Scope, counters: Resource<(), UserChatCounters>) -> impl IntoView {
    view! { cx, { move || {
        if let Some(counters) = counters.read(cx) {
            return view! { cx,
            <div class="card bg-darker p-3">
                <h2>"Counters:"</h2>
                <div class="card-body d-flex flex-column">
                <table style="color: rgba(0,0,0,0) !important;" class="table table-hover table-responsive text-white">
                    <thead>
                    <tr>
                        <th scope="col"></th>
                        <th scope="col"></th>
                    </tr>
                    </thead>
                    <tbody class="text-start">
                        <tr class="text-white no-border">
                            <td>"Chat count:"</td>
                            <td class="text-airtifex">{counters.chat_count}</td>
                        </tr>
                    </tbody>
                </table>
                </div>
            </div>
            }.into_view(cx)
       }
       view!{ cx, <></>}.into_view(cx)
    }}}
    .into_view(cx)
}
