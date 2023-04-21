use crate::components::status_message::*;
use crate::{api, web_util, Page, PageStack};
use airtifex_core::llm::PromptInspect;

use leptos::*;
use leptos_router::*;

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

    view! {cx, {move || {
      page_stack.update(|p| p.push(Page::PromptList));
      view! { cx,
        <div class="card-body d-flex flex-column px-5 pb-5">
          <table class="table table-hover table-striped table-responsive text-white">
            <thead>
            <tr>
              <th scope="col">"Prompt"</th>
              <th scope="col">"Response"</th>
              <th scope="col">"Model"</th>
              <th scope="col">"Date"</th>
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
    view! {cx, <tr
                class="text-white border-lighter"
              >
                  <td>{web_util::display_limited_str(&prompt.prompt, 50)}</td>
                  <td>{web_util::display_limited_str(&prompt.response, 50)}</td>
                  <td>{prompt.model}</td>
                  <td>{prompt.date.format("%a, %d %b %Y %H:%M:%S").to_string()}</td>
                  <td>
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
                                let navigate = use_navigate(cx);
                                navigate(&open_href, Default::default()).expect("prompt page");
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
