use crate::components::{status_message::*, titled_child_page::*};
use crate::{api, web_util, Page, PageStack};

use leptos::*;
use leptos_router::*;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct PromptParams {
    prompt_id: Option<String>,
}

#[component]
pub fn PromptView(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let params = use_params::<PromptParams>(cx);
    let status_message = create_rw_signal(cx.clone(), Message::Empty);
    let window_size = web_util::WindowSize::signal(cx).expect("windows size");
    let is_details_open = create_rw_signal(cx.clone(), false);

    let prompt_id = Signal::derive(cx, move || params.get().ok().and_then(|p| p.prompt_id));

    let prompt = create_resource(
        cx,
        move || prompt_id.get(),
        move |id| async move {
            match (authorized_api.get(), id) {
                (Some(api), Some(id)) => match api.prompt_inspect(&id).await {
                    Ok(prompt) => Some(prompt),
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

    let model = Signal::derive(cx, move || {
        if let Some(Some(prompt)) = prompt.read(cx) {
            prompt.model.clone()
        } else {
            Default::default()
        }
    });

    let title = Signal::derive(cx, move || {
        if let Some(Some(prompt)) = prompt.read(cx) {
            web_util::display_limited_str(&prompt.prompt, 50)
        } else {
            Default::default()
        }
    });

    let details = move || {
        if let Some(Some(prompt)) = prompt.read(cx) {
            if is_details_open.get() {
                return view!{ cx,
                <div class="card bg-darker">
                    <div class="card-body d-flex flex-row">
                        <table class="table table-hover table-responsive text-white mb-0">
                            <tbody>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Batch size: "</td>
                                    <td class="text-airtifex-yellow text-center">{prompt.n_batch}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Top K: "</td>
                                    <td class="text-airtifex-yellow text-center">{prompt.top_k}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Top P: "</td>
                                    <td class="text-airtifex-yellow text-center">{prompt.top_k}</td>
                                </tr>
                            </tbody>
                        </table>
                        <table class="table table-hover table-responsive text-white">
                            <tbody>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Date: "</td>
                                    <td class="text-secondary text-center">{prompt.date.format("%a, %d %b %Y %H:%M:%S").to_string()}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Repeat penalty: "</td>
                                    <td class="text-airtifex-yellow text-center">{prompt.repeat_penalty}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Temperature: "</td>
                                    <td class="text-airtifex-yellow text-center">{prompt.temp}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
                }.into_view(cx);
            }
        }
        view! { cx, <></> }.into_view(cx)
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::PromptView));
        let details_icon = if is_details_open.get() {
            "/icons/minus-circle.svg"
        } else {
            "/icons/plus-circle.svg"
        };

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-3 h-100 overflow-scroll" >
             <TitledChildPage title=title page_stack={page_stack.read_only()}></TitledChildPage>
             <div class="text-center w-100">
                 <p class="text-airtifex-light font-monospace">{model}</p>
             </div>
             <button
                class="btn-btn-airtifex btn-outline rounded me-auto ms-2 mb-2"
                on:click=move|_|is_details_open.update(|o| *o = !*o)
             >
              <img class="me-2" src=details_icon />
              "Details"
             </button>
             {details}
             { move || {
                if let Some(Some(prompt)) = prompt.read(cx) {
                    let (card_classes, classes) = if window_size.get().width < 992 {
                        ("card bg-darker col-12 p-3", "d-flex flex-column w-100")
                    } else {
                        ("card bg-darker col-6 p-3", "d-flex flex-row justify-content-center w-100")
                    };
                    view!{cx,
                        <div class=classes>
                            <div class=card_classes>
                                <div class="card-body">
                                    <h2 class="mb-4">"Prompt:"</h2>
                                    <pre>
                                        {prompt.prompt}
                                    </pre>
                                </div>
                            </div>
                            <div class=card_classes>
                                <div class="card-body">
                                    <h2 class="mb-4">"Response:"</h2>
                                    <pre>
                                        {prompt.response}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    }.into_view(cx)
                } else {
                    view!{cx, <></>}.into_view(cx)
                }
             }}
           </main>
        }.into_view(cx)
     }}
    }
}
