use email_address::EmailAddress;
use leptos::*;

#[component]
pub fn EmailValidation(
    cx: Scope,
    email: ReadSignal<String>,
    is_ok: RwSignal<bool>,
) -> impl IntoView {
    create_effect(cx, move |_| {
        is_ok.update(|is| *is = EmailAddress::is_valid(&email.get()));
    });

    view! { cx,
      {move || {
        if is_ok.get() {
            view!{ cx,
                 <p class="text-airtifex-green">"Email valid"</p>
            }.into_view(cx)
        } else {
            view!{ cx,
                 <p class="text-airtifex-red">"Email invalid"</p>
            }.into_view(cx)
        }
       }}
    }
    .into_view(cx)
}
