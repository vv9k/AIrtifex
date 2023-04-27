use crate::{api, components::status_message::*, pages, web_util, Page, PageStack};
use airtifex_core::llm::PromptInspect;

use leptos::*;

#[component]
pub fn PromptList(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let status_message = create_rw_signal(cx, Message::Empty);
    let remove_prompt_id = create_rw_signal(cx, None);

    let prompts = create_resource(
        cx,
        move || (),
        move |_| async move {
            match authorized_api.get() {
                Some(api) => match api.prompt_list().await {
                    Ok(prompts) => prompts,
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

    view! {cx, {move || {
      page_stack.update(|p| p.push(Page::PromptList));
      view! { cx,
        <main class="bg-dark text-white d-flex flex-column p-3 overflow-auto" >
            <div class="card bg-darker">
                <div class="card-body d-flex flex-column">
                <table class="table table-hover table-striped table-responsive text-white">
                    <thead>
                    <tr>
                    <th scope="col">"Prompt"</th>
                    <th scope="col">"Response"</th>
                    <th class="text-center" scope="col">"Model"</th>
                    <th class="text-center" scope="col">"Date"</th>
                    <th scope="col">""</th>
                    </tr>
                    </thead>
                    <tbody>
                    {
                    prompts.read(cx).unwrap_or_default().into_iter().map(|prompt| {
                        view!{cx, <PromptListEntry prompt remove_prompt_id />}.into_view(cx)
                    }).collect::<Vec<_>>()
                    }
                    </tbody>
                </table>
                </div>
            </div>
        </main>
      }.into_view(cx)
    }}}
    .into_view(cx)
}

#[component]
fn PromptListEntry(
    cx: Scope,
    prompt: PromptInspect,
    remove_prompt_id: RwSignal<Option<String>>,
) -> impl IntoView {
    let open_href = format!("/prompt/{}", prompt.id);
    let open_href2 = open_href.clone();

    let window_size = web_util::WindowSize::signal(cx).expect("window size");
    let char_count = Signal::derive(cx, move || {
        let size = window_size.get();
        ((size.width / 64) + 20).max(30) as usize
    });

    view! {cx, <tr
                class="text-white no-border align-middle"
              >
                  <td
                    style="cursor: pointer;"
                    on:click=move |_| {
                        pages::goto(cx, &open_href2).expect("prompt page");
                    }
                  >{move || web_util::display_limited_str(&prompt.prompt, char_count.get())}</td>
                  <td class="fst-italic">{move || web_util::display_limited_str(&prompt.response, char_count.get())}</td>
                  <td align="center" class="text-airtifex-light">{prompt.model}</td>
                  <td align="center" class="text-secondary">{prompt.date.format("%a, %d %b %Y %H:%M:%S").to_string()}</td>
                  <td align="right">
                      <div class="btn-group" role="prompt toolbar" aria-label="prompt toolbar">
                          <button
                            class="btn btn-outline-lighter"
                            data-bs-toggle="modal"
                            data-bs-target="#removePromptModal"
                            on:focus=move |_| {
                                remove_prompt_id.update(|c| *c = Some(prompt.id.clone()));
                            }
                          >
                              <img src="/icons/minus-circle.svg" class="me-2" />
                              "Remove"
                          </button>
                          <button
                            class="btn btn-outline-lighter"
                            on:click=move |_| {
                                pages::goto(cx, &open_href).expect("prompt page");
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
