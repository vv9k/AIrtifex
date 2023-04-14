use leptos::*;

#[component]
pub fn ListPageControl(
    cx: Scope,
    current_list_page: RwSignal<u32>,
    elem_count: Signal<usize>,
    page_size: ReadSignal<usize>,
) -> impl IntoView {
    let back_page_btn = move || {
        if current_list_page.get() > 1 {
            view! { cx,
              <button
                  class="pe-3"
                  on:click=move |_| current_list_page.update(|p| if *p > 1 {*p -= 1; })
              >
                  <img src="/icons/arrow-left-circle.svg" />
              </button>
            }
            .into_view(cx)
        } else {
            view! { cx, <></> }.into_view(cx)
        }
    };

    let page = move || {
        if elem_count.get() == page_size.get() {
            format!("{}/...", current_list_page.get())
        } else {
            format!("{0}/{0}", current_list_page.get())
        }
    };

    let forward_page_btn = move || {
        if elem_count.get() == page_size.get() {
            view! { cx,
              <button
                  class="ps-3"
                  on:click=move |_| current_list_page.update(|p| *p += 1)
              >
                  <img src="/icons/arrow-right-circle.svg" />
              </button>
            }
            .into_view(cx)
        } else {
            view! { cx, <></> }.into_view(cx)
        }
    };

    view! { cx,
        <div class="d-flex flex-row justify-content-center pt-3">
            {back_page_btn}
            {page}
            {forward_page_btn}
        </div>
    }
    .into_view(cx)
}
