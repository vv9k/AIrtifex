use crate::components::go_back_button::*;

use leptos::*;

#[component]
pub fn TitledChildPage<'a>(
    cx: Scope,
    title: Signal<String>,
    parent_href: &'a str,
) -> impl IntoView {
    view! { cx,
         <div class="d-flex flex-column justify-content-start">
             <GoBackButton href=parent_href></GoBackButton>
             <h1 class="display-5 p-1 pt-2 text-center">{title}</h1>
         </div>
    }
}
