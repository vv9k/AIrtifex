use leptos::*;

#[component]
pub fn PasswordValidation(
    cx: Scope,
    password: ReadSignal<String>,
    confirm_password: Option<ReadSignal<String>>,
    is_ok: RwSignal<bool>,
) -> impl IntoView {
    let has_lowercase_letter = Signal::derive(cx, move || {
        let password = password.get();
        password.chars().any(|c| c.is_ascii_lowercase())
    });
    let has_uppercase_letter = Signal::derive(cx, move || {
        let password = password.get();
        password.chars().any(|c| c.is_ascii_uppercase())
    });
    let has_number = Signal::derive(cx, move || {
        let password = password.get();
        password.chars().any(|c| c.is_numeric())
    });
    let has_correct_len = Signal::derive(cx, move || {
        let password = password.get();
        password.len() >= 8
    });
    let password_match = Signal::derive(cx, move || {
        let password = password.get();
        confirm_password.map(|confirm| confirm.get() == password)
    });

    create_effect(cx, move |_| {
        if let Some(password_match) = password_match.get() {
            is_ok.update(|is| {
                *is = has_lowercase_letter.get()
                    && has_uppercase_letter.get()
                    && has_number.get()
                    && has_correct_len.get()
                    && password_match
            })
        } else {
            is_ok.update(|is| {
                *is = has_lowercase_letter.get()
                    && has_uppercase_letter.get()
                    && has_number.get()
                    && has_correct_len.get()
            })
        }
    });

    view!{ cx, {move || {
        let lower_letter_class = move || if has_lowercase_letter.get() { "text-airtifex-green" } else { "text-airtifex-red" };
        let upper_letter_class = move || if has_uppercase_letter.get() { "text-airtifex-green" } else { "text-airtifex-red" };
        let number_class = move || if has_number.get() { "text-airtifex-green" } else { "text-airtifex-red" };
        let len_class = move || if has_correct_len.get() { "text-airtifex-green" } else { "text-airtifex-red" };
        let match_elem = move || if let Some(password_match) = password_match.get() {
            let class = if password_match { "text-airtifex-green" } else {"text-airtifex-red"};
            view!{ cx, <p class=class>"Passwords must "<b>"match"</b></p> }.into_view(cx)
        } else {
            view!{ cx, <></>}.into_view(cx)
        };
        view!{ cx,
           <div>
             <p class=lower_letter_class>"A "<b>"lowercase "</b> "letter"</p>
             <p class=upper_letter_class>"A "<b>"capital (uppercase) "</b>" letter"</p>
             <p class=number_class>"A "<b>"number"</b></p>
             <p class=len_class>"Minimum "<b>"8 characters"</b></p>
             { match_elem }
           </div>
        }.into_view(cx)
       }}
    }.into_view(cx)
}
