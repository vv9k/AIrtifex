use crate::{pages, Page};

use airtifex_core::image::ImageInspect;
use leptos::*;

#[component]
pub fn RecentImages(cx: Scope, images: Resource<(), Vec<ImageInspect>>) -> impl IntoView {
    const DISPLAY_COUNT: usize = 5;

    view! { cx, { move || {
        let view_all_btn = view! { cx,
            <button
                class="btn btn-outline-lighter round mx-auto"
                on:click={move |_| {
                pages::goto(cx, Page::GenerateImage).expect("images page");
                }}
            >
                <img src="/icons/list.svg" class="me-2" />
                "View All"
            </button>
        }
        .into_view(cx);

        if let Some(images) = images.read(cx) {
            let inner_view = images
                .into_iter()
                .take(DISPLAY_COUNT)
                .map(|image| view! {cx, <RecentImageEntry image />}.into_view(cx))
                .collect::<Vec<_>>();

            let view_all_btn = if inner_view.len() > DISPLAY_COUNT {
                view_all_btn
            } else {
                view! {cx, <></>}.into_view(cx)
            };
            return view! { cx,
                <div class="card bg-darker p-3 col-12">
                    <h2>"Recent Images"</h2>
                    <div class="card-body d-flex flex-column">
                    <table style="color: rgba(0,0,0,0) !important;" class="table table-hover table-responsive text-white">
                        <thead>
                        <tr>
                        <th scope="col">""</th>
                        <th scope="col">""</th>
                        <th scope="col">""</th>
                        </tr>
                        </thead>
                        <tbody>
                            {inner_view}
                        </tbody>
                    </table>
                    {view_all_btn}
                    </div>
                </div>
                }.into_view(cx)
       }
       view!{ cx, <></>}.into_view(cx)
    }}}
    .into_view(cx)
}

#[component]
fn RecentImageEntry(cx: Scope, image: ImageInspect) -> impl IntoView {
    let view_href = format!("{}/{}", Page::GenerateImage.raw_path(), image.id);
    let is_finished = if !image.processing {
        view! { cx, <span class="text-airtifex-green">"✓"</span>}
    } else {
        view! { cx, <span class="text-airtifex-yellow">"✗"</span>}
    };
    view! {cx, <tr
                class="text-white no-border"
                style="cursor: pointer;"
                on:click={move |_| {
                    pages::goto(cx, &view_href).expect("image page");
                }}
              >
                  <td>{is_finished}</td>
                  <td class="text-start">{image.prompt}</td>
                  <td class="text-airtifex-light text-center">{image.model}</td>
              </tr>
    }
    .into_view(cx)
}
