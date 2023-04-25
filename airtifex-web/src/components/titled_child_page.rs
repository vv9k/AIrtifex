use crate::components::go_back_button::*;

use leptos::*;

#[component]
pub fn TitledChildPage(cx: Scope, title: Signal<String>) -> impl IntoView {
    view! { cx,
         <div class="d-flex flex-column justify-content-start">
             <GoBackButton></GoBackButton>
             <h1 class="display-5 p-1 pt-2 text-center">{title}</h1>
         </div>
    }
}
