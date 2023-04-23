use crate::{
    api,
    components::{
        email_validation::*, password_validation::*, status_message::*, titled_child_page::*,
        users::account_type_selector::*,
    },
    pages, Page, PageStack,
};
use airtifex_core::user::{AccountType, UserRegisterRequest};

use leptos::*;

#[component]
pub fn UserAdd(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    page_stack: RwSignal<PageStack>,
    users_message: RwSignal<Message>,
) -> impl IntoView {
    let password = create_rw_signal(cx, String::new());
    let (username, set_username) = create_signal(cx, String::new());
    let email = create_rw_signal(cx, String::new());
    let (account_type, set_account_type) = create_signal(cx, AccountType::User);
    let is_pass_ok = create_rw_signal(cx, false);
    let is_email_ok = create_rw_signal(cx, false);

    let is_button_disabled =
        move || !is_pass_ok.get() || !is_email_ok.get() || username.get().is_empty();

    let add_user_action = create_action(
        cx,
        move |(username, password, email, account_type): &(String, String, String, AccountType)| {
            let user = username.clone();
            let request = UserRegisterRequest {
                username: username.clone(),
                password: password.clone(),
                email: email.clone(),
                account_type: account_type.clone(),
            };
            async move {
                if let Some(api) = authorized_api.get() {
                    let response = api.user_add(request).await;

                    match response {
                        Ok(id) => {
                            pages::goto(cx, Page::Users.path()).expect("users page");
                            users_message.update(|m| {
                                *m = Message::Success(format!(
                                    "successfully created user {user}, id: {id}"
                                ));
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

    let dispatch_add_user = move || {
        add_user_action.dispatch((
            username.get(),
            password.get(),
            email.get(),
            account_type.get(),
        ))
    };

    view! { cx,
      {move || {
        page_stack.update(|p| p.push(Page::UserAdd));
        let title = Signal::derive(cx, move || "Add new user".into());

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
                             type = "text"
                             class = "form-control"
                             required
                             placeholder = "Username"
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
                       </div>
                       <div class="input-group mb-3">
                           <input
                             type = "text"
                             class = "form-control"
                             required
                             placeholder = "Email"
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
                           <input
                             type = "password"
                             class = "form-control"
                             required
                             placeholder = "Password"
                             on:keyup = move |ev: ev::KeyboardEvent| {
                               match &*ev.key() {
                                   "Enter" => {
                                       dispatch_add_user()
                                   }
                                   _=> {
                                      let val = event_target_value(&ev);
                                      password.update(|p|*p = val);
                                   }
                               }
                             }
                             // The `change` event fires when the browser fills the form automatically,
                             on:change = move |ev| {
                               let val = event_target_value(&ev);
                               password.update(|p|*p = val);
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
                               if let Some(account_type) = AccountType::from_str(val) {
                                   set_account_type.set(account_type);
                               }
                             }
                           >
                               <AccountTypeSelector selected_account_type=account_type account_type=AccountType::Admin></AccountTypeSelector>
                               <AccountTypeSelector selected_account_type=account_type account_type=AccountType::User></AccountTypeSelector>
                               <AccountTypeSelector selected_account_type=account_type account_type=AccountType::Service></AccountTypeSelector>
                           </select>
                       </div>
                       <button
                         class="btn btn-airtifex rounded mt-3"
                         prop:disabled = is_button_disabled
                         on:click = move |_| dispatch_add_user()
                       >
                       "Submit"
                       </button>
                   </form>
                   <StatusMessage message=users_message></StatusMessage>
                   <div class="mt-4">
                       <h5>"Validation:"</h5>
                       <PasswordValidation password=password.read_only() confirm_password=None is_ok=is_pass_ok />
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
