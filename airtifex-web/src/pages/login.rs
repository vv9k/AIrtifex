use crate::{
    api::{AuthorizedApi, UnauthorizedApi},
    components::{credentials::*, status_message::*},
};
use airtifex_core::auth::Credentials;

use leptos::*;

#[component]
pub fn Login<F>(cx: Scope, api: UnauthorizedApi, on_success: F) -> impl IntoView
where
    F: Fn(AuthorizedApi) + 'static + Clone,
{
    let message = create_rw_signal(cx, Message::Empty);
    let (wait_for_response, set_wait_for_response) = create_signal(cx, false);

    let login_action = create_action(cx, move |(username, password): &(String, String)| {
        log::debug!("Try to login with {username}");
        let credentials = Credentials::new(username, password);
        let on_success = on_success.clone();
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = api.login(&credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    message.update(|m| *m = Message::Empty);
                    on_success(res);
                }
                Err(err) => {
                    let msg = err.to_string();
                    error!("Unable to login with {}: {msg}", credentials.username());
                    message.update(|m| *m = Message::Error(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(cx, move || wait_for_response.get());

    view! { cx,
        <main class="bg-dark text-white form-signin" >
            <div class="container d-flex h-100 justify-content-center">
                <div class="row align-items-center h-100">
                    <div class="card bg-darker">
                        <div style="padding-left: 0 !important;padding-right: 0 !important;" class="card-body py-5">
                            <div class="col-md-8 mx-auto text-center py-5">
                                <h1 class="display-5 font-monospace">"Welcome to "<span class="fw-bold"><span class="text-airtifex">"AI"</span>"rtifex"</span></h1>
                                <CredentialsForm
                                title = "Please login to your account"
                                action_label = "Login"
                                action = login_action
                                message
                                disabled
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </main>
    }
}
