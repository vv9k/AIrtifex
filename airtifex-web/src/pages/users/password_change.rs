use crate::components::{password_validation::*, status_message::*, titled_child_page::*};
use crate::{api, pages, Page, PageStack};
use airtifex_core::user::PasswordChangeRequest;

use leptos::*;
use leptos_router::*;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct PasswordChangeParams {
    username: Option<String>,
}

#[component]
pub fn UserPasswordChange(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
    users_message: RwSignal<Message>,
) -> impl IntoView {
    let new_password = create_rw_signal(cx, String::new());
    let confirm_password = create_rw_signal(cx, String::new());
    let is_pass_ok = create_rw_signal(cx, false);

    let params = use_params::<PasswordChangeParams>(cx);

    let change_password_action =
        create_action(cx, move |(username, password): &(String, String)| {
            let username = username.clone();
            let request = PasswordChangeRequest {
                new_password: password.clone(),
            };
            async move {
                if let Some(api) = authorized_api.get() {
                    let response = api.user_change_password(&username, request).await;

                    match response {
                        Ok(_) => {
                            pages::goto(cx, Page::Users.path()).expect("users page");
                            users_message.update(|m| {
                                *m = Message::Success(format!(
                                    "Successfully changed password of {username}"
                                ));
                            });
                        }
                        Err(err) => users_message.update(|m| {
                            *m = Message::Error(format!(
                                "failed to change password of {username} - {err}"
                            ));
                        }),
                    }
                } else {
                    users_message.update(|m| {
                        *m = Message::Error("failed to connect to API".into());
                    });
                }
            }
        });

    let dispatch_change_password = move || {
        change_password_action.dispatch((
            params
                .get()
                .ok()
                .and_then(|p| p.username)
                .unwrap_or_default(),
            new_password.get(),
        ))
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::UserAdd));
        let title = Signal::derive(cx, move || format!("Change password for user {}", params.get().ok().and_then(|p| p.username).unwrap_or_default()));

        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3" >
             <TitledChildPage title=title page_stack={page_stack.read_only()}></TitledChildPage>
             <div class="d-flex justify-content-center flex-column h-100">
               <div class="container-fluid col-9 col-lg-4">
                 <div class="card bg-darker p-5">
                   <div class="card-body">
                   <form
                     on:submit=|ev|ev.prevent_default()
                     class="row text-start"
                   >
                       <div class="input-group mb-3">
                           <input
                             type = "password"
                             class = "form-control"
                             required
                             placeholder = "New password..."
                             on:keyup = move |ev: ev::KeyboardEvent| {
                               let val = event_target_value(&ev);
                               new_password.update(|v|*v = val);
                             }
                             // The `change` event fires when the browser fills the form automatically,
                             on:change = move |ev| {
                               let val = event_target_value(&ev);
                               new_password.update(|v|*v = val);
                             }
                           />
                       </div>
                       <div class="input-group mb-3">
                           <input
                             type = "password"
                             class = "form-control"
                             required
                             placeholder = "Confirm password..."
                             on:keyup = move |ev: ev::KeyboardEvent| {
                               let val = event_target_value(&ev);
                               confirm_password.update(|v|*v = val);
                             }
                             // The `change` event fires when the browser fills the form automatically,
                             on:change = move |ev| {
                               let val = event_target_value(&ev);
                               confirm_password.update(|v|*v = val);
                             }
                           />
                       </div>
                       <div class="mt-4">
                           <h5>"Validation:"</h5>
                           <PasswordValidation password=new_password.read_only() confirm_password=Some(confirm_password.read_only()) is_ok=is_pass_ok />
                       </div>
                       <button
                         class="btn btn-outline-lighter rounded mt-3"
                         prop:disabled = move || !is_pass_ok.get()
                         on:click = move |_| dispatch_change_password()
                       >
                       <img class="me-2" src="/icons/check.svg" />
                       "Submit"
                       </button>
                   </form>

                   <StatusMessage message=users_message></StatusMessage>
                   </div>
                 </div>
               </div>
             </div>
           </main>
        }.into_view(cx)
     }}
    }
}
