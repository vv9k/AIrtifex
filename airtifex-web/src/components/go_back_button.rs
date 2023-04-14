use leptos::*;

#[component]
pub fn GoBackButton<'a>(cx: Scope, href: &'a str) -> impl IntoView {
    view! { cx,
      <a class="btn btn-outline-lighter me-auto ms-2" href=href>
          <img class="fill-airtifex me-2" src="/icons/arrow-left-circle.svg" />
          "Go back"
      </a>
    }
}
