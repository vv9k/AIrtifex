use crate::pages::PageStack;

use leptos::*;

#[component]
pub fn GoBackButton(cx: Scope) -> impl IntoView {
    let href = move || PageStack::load().parent().path();
    view! { cx,
      <a class="btn btn-outline-lighter me-auto ms-2" href=href>
          <img class="fill-airtifex me-2" src="/icons/arrow-left-circle.svg" />
          "Go back"
      </a>
    }
}
