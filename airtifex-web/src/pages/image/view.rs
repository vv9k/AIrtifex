use crate::components::{status_message::*, titled_child_page::*};
use crate::{api, Page, PageStack};

use base64::engine::Engine;
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

    let images = create_resource(
        cx,
        move || dummy_images_signal.get(),
        move |_| async move {
            match (authorized_api.get(), image_id.get()) {
                (Some(api), Some(id)) => match api.image_samples(&id).await {
                    Ok(images) => Some(images),
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
                 <table class="table table-hover table-striped table-responsive text-white">
                   <thead>
                    <tr>
                     <th scope="col" class="border-0 font-monospace">"Finished: "{is_finished}</th>
                     <th scope="col" class="border-0 font-monospace">"Width: "<span class="text-airtifex-yellow">{metadata.width}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Height: "<span class="text-airtifex-yellow">{metadata.height}</span></th>
                     <th scope="col" class="border-0 font-monospace">"N Steps: "<span class="text-airtifex-yellow">{metadata.n_steps}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Seed: "<span class="text-airtifex-yellow">{metadata.seed}</span></th>
                     <th scope="col" class="border-0 font-monospace">"N Samples: "<span class="text-airtifex-yellow">{metadata.num_samples}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Guidance Scale: "<span class="text-airtifex-yellow">{metadata.guidance_scale}</span></th>
                     <th scope="col" class="border-0 font-monospace">"Created Date: "<span class="text-airtifex-yellow">{metadata.create_date.format("%a, %d %b %Y %H:%M:%S").to_string()}</span></th>
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
        page_stack.update(|p| p.push(Page::GeneratedImageView));
        let engine = base64::engine::GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            base64::engine::general_purpose::PAD,
        );


        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3 overflow-auto" >
             <TitledChildPage title={image_id} parent_href=Page::GenerateImage.path()></TitledChildPage>
             <div class="text-center w-100">
                 <p class="text-airtifex-yellow font-monospace py-1">{model}</p>
                 <p class="text-airtifex-light font-monospace py-2">{prompt}</p>
             </div>
             {details}
             <div class="mx-auto p-3">
             {move || {
                let size = size.get();
                if let Some(Some(images)) = images.read(cx) {
                     images.into_iter().map(|i| {
                        let encoded = engine.encode(&i.data);
                        let src = format!("data:image/png;base64,{encoded}");
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
