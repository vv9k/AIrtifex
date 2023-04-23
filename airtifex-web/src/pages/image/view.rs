use crate::components::{status_message::*, titled_child_page::*};
use crate::{api, pages, web_util, Page, PageStack};

use leptos::*;
use leptos_router::*;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct ImageParams {
    image_id: Option<String>,
}

#[component]
pub fn ImageView(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
) -> impl IntoView {
    let params = use_params::<ImageParams>(cx);

    let dummy_images_signal = create_rw_signal::<u32>(cx, 1);
    let status_message = create_rw_signal(cx, Message::Empty);
    let is_details_open = create_rw_signal(cx, true);

    let image_id = Signal::derive(cx, move || params.get().ok().and_then(|p| p.image_id));

    let metadata = create_resource(
        cx,
        move || dummy_images_signal.get(),
        move |_| async move {
            match (authorized_api.get(), image_id.get()) {
                (Some(api), Some(id)) => match api.image_info(&id).await {
                    Ok(meta) => Some(meta),
                    Err(e) => {
                        let e = e.to_string();
                        pages::goto_login_if_expired(cx, &e, authorized_api);
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

    let images = create_resource(
        cx,
        move || dummy_images_signal.get(),
        move |_| async move {
            match (authorized_api.get(), image_id.get()) {
                (Some(api), Some(id)) => match api.image_samples(&id).await {
                    Ok(images) => Some(images),
                    Err(e) => {
                        let e = e.to_string();
                        pages::goto_login_if_expired(cx, &e, authorized_api);
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

    let image_id = Signal::derive(cx, move || {
        metadata
            .read(cx)
            .and_then(|m| m.map(|m| m.id))
            .unwrap_or("Image".into())
    });
    let prompt = Signal::derive(cx, move || {
        metadata
            .read(cx)
            .and_then(|m| m.map(|m| m.prompt))
            .unwrap_or_default()
    });
    let model = Signal::derive(cx, move || {
        metadata
            .read(cx)
            .and_then(|m| m.map(|m| m.model))
            .unwrap_or_default()
    });
    let size = Signal::derive(cx, move || {
        metadata
            .read(cx)
            .and_then(|m| m.map(|m| (m.width, m.height)))
            .unwrap_or((256, 256))
    });

    let details = move || {
        if let Some(Some(metadata)) = metadata.read(cx) {
            let icon = if is_details_open.get() {
                "/icons/minus-circle.svg"
            } else {
                "/icons/plus-circle.svg"
            };
            let is_finished = if !metadata.processing {
                view! { cx, <span class="text-airtifex-green">"✓"</span>}
            } else {
                view! { cx, <span class="text-airtifex-yellow">"✗"</span>}
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
                 <div class="card bg-darker">
                    <div class="card-body d-flex flex-row">
                        <table class="table table-hover table-responsive text-white mb-0">
                            <tbody>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Finished: "</td>
                                    <td class="text-airtifex-yellow text-center">{is_finished}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Width: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.width}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Height: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.height}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Guidance Scale: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.guidance_scale}</td>
                                </tr>
                            </tbody>
                        </table>
                        <table class="table table-hover table-responsive text-white">
                            <tbody>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Created Date: "</td>
                                    <td class="text-secondary text-center">{metadata.create_date.format("%a, %d %b %Y %H:%M:%S").to_string()}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"N Steps: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.n_steps}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"N Samples: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.num_samples}</td>
                                </tr>
                                <tr class="no-border">
                                    <td class="fitwidth text-white">"Seed: "</td>
                                    <td class="text-airtifex-yellow text-center">{metadata.seed}</td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                 </div>
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
        page_stack.update(|p| p.push(Page::GeneratedImageView));


        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-3 overflow-auto" >
             <TitledChildPage title={image_id} page_stack={page_stack.read_only()}></TitledChildPage>
             <div class="text-center w-100">
                 <p class="text-airtifex-light font-monospace py-1">{model}</p>
                 <p class="text-airtifex-yellow font-monospace py-2">{prompt}</p>
             </div>
             {details}
             {move || {
                if let Some(Some(metadata)) = metadata.read(cx) {
                    if let Some(input_image) = metadata.input_image {
                        let src= web_util::encode_image_base64(&input_image);
                        let size = size.get();

                        let mask = if let Some(mask) = metadata.mask {
                            let src= web_util::encode_image_base64(&mask);

                            view!{cx,
                                <div class="d-flex flex-column">
                                    <h2>"Mask:"</h2>
                                    <img class="p-2" src=src width=size.0 height=size.1></img>
                                </div>
                            }.into_view(cx)
                        } else {
                            view! {cx, <></> }.into_view(cx)
                        };

                        return view!{cx,
                            <div class="mx-auto p-3">
                                <div class="d-flex flex-column">
                                    <h2>"Input Image:"</h2>
                                    <img class="p-2" src=src width=size.0 height=size.1></img>
                                </div>
                                {mask}
                            </div>
                        }.into_view(cx);
                    }
                }
                view! {cx, <></> }.into_view(cx)
             }}
             <div class="mx-auto p-3">
                <h2>"Generated images:"</h2>
             {move || {
                let size = size.get();
                if let Some(Some(images)) = images.read(cx) {
                     images.into_iter().map(|i| {
                        let src= web_util::encode_image_base64(&i.data);
                        view!{cx, <img class="p-2" src=src width=size.0 height=size.1></img>}.into_view(cx)
                    }).collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }}
            </div>
           </main>
        }.into_view(cx)
     }}
    }
}
