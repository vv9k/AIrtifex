use airtifex_core::user::AccountType;

use leptos::*;

#[component]
pub fn AccountTypeSelector(
    cx: Scope,
    selected_account_type: ReadSignal<AccountType>,
    account_type: AccountType,
) -> impl IntoView {
    let value = account_type.to_str();
    let is_selected = Signal::derive(cx, move || selected_account_type.get() == account_type);
    view! { cx,
        {move || if is_selected.get() {
            view!{ cx,
                <option value=value selected>{value}</option>
            }.into_view(cx)
        } else {
            view!{ cx,
                <option value=value>{value}</option>
            }.into_view(cx)
        }}
    }
    .into_view(cx)
}
