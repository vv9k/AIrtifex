use crate::components::status_message::*;

use leptos::{ev, *};

#[component]
pub fn CredentialsForm(
    cx: Scope,
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String), ()>,
    message: RwSignal<Message>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = create_signal(cx, String::new());
    let (username, set_username) = create_signal(cx, String::new());

    let dispatch_action = move || action.dispatch((username.get(), password.get()));

    let button_is_disabled = Signal::derive(cx, move || {
        disabled.get() || password.get().is_empty() || username.get().is_empty()
    });

    view! { cx,
      <p class="pt-5">{ title }</p>
      <StatusMessage message></StatusMessage>
      <form class="row text-start pt-2 px-5" on:submit=|ev|ev.prevent_default()>
        <input
          class = "form-control"
          type = "text"
          required
          placeholder = "Username"
          prop:disabled = move || disabled.get()
          on:keyup = move |ev: ev::KeyboardEvent| {
            let val = event_target_value(&ev);
            set_username.update(|v|*v = val);
          }
          // The `change` event fires when the browser fills the form automatically,
          on:change = move |ev| {
            let val = event_target_value(&ev);
            set_username.update(|v|*v = val);
          }
        />
        <input
          class = "form-control"
          type = "password"
          required
          placeholder = "Password"
          prop:disabled = move || disabled.get()
          on:keyup = move |ev: ev::KeyboardEvent| {
            match &*ev.key() {
                "Enter" => {
                   dispatch_action();
                }
                _=> {
                   let val = event_target_value(&ev);
                   set_password.update(|p|*p = val);
                }
            }
          }
          // The `change` event fires when the browser fills the form automatically,
          on:change = move |ev| {
            let val = event_target_value(&ev);
            set_password.update(|p|*p = val);
          }
        />
        <button
          class="btn btn-outline-lighter rounded mt-3"
          prop:disabled = move || button_is_disabled.get()
          on:click = move |_| dispatch_action()
        >
        <img class="me-2" src="/icons/log-in.svg" />
        {
            action_label
        }
        </button>
      </form>
    }
}
