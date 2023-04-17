use crate::components::{
    email_validation::*, status_message::*, titled_child_page::*, users::account_type_selector::*,
};
use crate::{api, Page, PageStack};
use airtifex_core::user::{AccountType, UserEditRequest};

use leptos::*;
use leptos_router::*;

#[derive(Params, PartialEq, Clone, Debug)]
pub struct EditParams {
    pub username: Option<String>,
    pub parent: Option<String>,
}

#[component]
pub fn UserEdit(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
    users_message: RwSignal<Message>,
) -> impl IntoView {
    let params = use_params::<EditParams>(cx);

    let username = Signal::derive(cx, move || {
        params
            .get()
            .ok()
            .and_then(|p| p.username)
            .unwrap_or_default()
    });
    let email = create_rw_signal(cx, String::new());
    let is_email_ok = create_rw_signal(cx, false);
    let account_type = create_rw_signal(cx, AccountType::User);

    let _ = create_resource(
        cx,
        move || username.get(),
        move |username| async move {
            match authorized_api.get() {
                Some(api) => match api.user_info(&username).await {
                    Ok(user) => {
                        email.set(user.email.clone());
                        account_type
                            .update(|account_type| *account_type = user.account_type.clone());
                        Some(user)
                    }
                    Err(e) => {
                        let e = e.to_string();
                        crate::pages::goto_login_if_expired(cx, &e, authorized_api);
                        users_message.update(|msg| *msg = Message::Error(e));
                        None
                    }
                },
                None => {
                    users_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    None
                }
            }
        },
    );

    let is_button_disabled = move || !is_email_ok.get();

    let edit_user_action = create_action(
        cx,
        move |(username, email, account_type): &(String, String, AccountType)| {
            let user = username.clone();
            let request = UserEditRequest {
                email: email.clone(),
                account_type: account_type.clone(),
            };
            async move {
                if let Some(api) = authorized_api.get() {
                    let response = api.user_edit(&user, request).await;

                    match response {
                        Ok(_) => {
                            users_message.update(|m| {
                                *m = Message::Success(format!("successfully edited user {user}"));
                            });
                        }
                        Err(err) => users_message.update(|m| {
                            *m = Message::Error(format!("failed to create user - {err}"));
                        }),
                    }
                } else {
                    users_message.update(|m| {
                        *m = Message::Error("failed to connect to API".into());
                    });
                }
            }
        },
    );

    let dispatch_edit_user = move || {
        edit_user_action.dispatch((username.get(), email.get(), account_type.get()));
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::UserEdit));
        let title = Signal::derive(cx, move || format!("Edit user {}", username.get()));
        view!{cx,
           <main class="bg-dark text-white d-flex flex-column p-1 pt-3" >
             <TitledChildPage title=title parent_href=page_stack.get().parent().path()></TitledChildPage>

             <div class="d-flex justify-content-center flex-column h-100">
               <div class="container-fluid col-9 col-lg-4">
                 <div class="card bg-darker p-5">
                   <div class="card-body">
                     <form
                       on:submit=|ev|ev.prevent_default()
                       class="row text-start mb-3"
                     >
                         <div class="input-group mb-3">
                             <label class="input-group-text" for="emailInput">"Email"</label>
                             <input
                               id = "emailInput"
                               type = "text"
                               class = "form-control"
                               required
                               placeholder = "..."
                               value=move || email.get()
                               on:keyup = move |ev: ev::KeyboardEvent| {
                                 let val = event_target_value(&ev);
                                 email.update(|v|*v = val);
                               }
                               // The `change` event fires when the browser fills the form automatically,
                               on:change = move |ev| {
                                 let val = event_target_value(&ev);
                                 email.update(|v|*v = val);
                               }
                             />
                         </div>
                         <div class="input-group mb-3">
                             <label class="input-group-text" for="accountTypeSelect">"Account type"</label>
                             <select
                               class="form-select"
                               id="accountTypeSelect"
                               name="account_type"
                               on:change = move |ev| {
                                 let val = event_target_value(&ev);
                                 if let Some(acc_type) = AccountType::from_str(val) {
                                     account_type.update(|a| *a = acc_type);
                                 }
                               }
                             >
                                 <AccountTypeSelector selected_account_type=account_type.read_only() account_type=AccountType::Admin></AccountTypeSelector>
                                 <AccountTypeSelector selected_account_type=account_type.read_only() account_type=AccountType::User></AccountTypeSelector>
                                 <AccountTypeSelector selected_account_type=account_type.read_only() account_type=AccountType::Service></AccountTypeSelector>
                             </select>
                         </div>
                         <button
                           class="btn btn-outline-lighter rounded mt-3"
                           prop:disabled = is_button_disabled
                           on:click = move |_| dispatch_edit_user()
                         >
                         <img class="me-2" src="/icons/check.svg" />
                         "Submit"
                         </button>
                     </form>
                     <StatusMessage message=users_message></StatusMessage>

                     <div class="mt-4">
                         <h5>"Validation:"</h5>
                         <EmailValidation email=email.read_only() is_ok=is_email_ok />
                     </div>
                   </div>
                 </div>
               </div>
             </div>
           </main>
        }.into_view(cx)
     }}
    }
}
