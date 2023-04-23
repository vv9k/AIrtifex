use crate::{components::go_back_button::*, pages::PageStack};

use leptos::*;

#[component]
pub fn TitledChildPage(
    cx: Scope,
    title: Signal<String>,
    page_stack: ReadSignal<PageStack>,
) -> impl IntoView {
    let last = page_stack.get().parent().path();
    view! { cx,
         <div class="d-flex flex-column justify-content-start">
             <GoBackButton href=last></GoBackButton>
             <h1 class="display-5 p-1 pt-2 text-center">{title}</h1>
         </div>
    }
}
