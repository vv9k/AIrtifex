use crate::{
    api,
    components::{list_page_control::*, modal::*, status_message::*, users::list_entry::*},
    pages::goto_login_if_expired,
    Page,
};

use airtifex_core::user::ListQuery;
use leptos::*;

pub mod add;
pub mod edit;
pub mod password_change;
pub mod profile;

pub use add::*;
pub use edit::*;
pub use password_change::*;
pub use profile::*;

#[component]
pub fn Users(
    cx: Scope,
    authorized_api: RwSignal<Option<api::AuthorizedApi>>,
    users_message: RwSignal<Message>,
) -> impl IntoView {
    let current_list_page = create_rw_signal::<u32>(cx, 1);
    let page_size = create_rw_signal::<usize>(cx, 25);
    let remove_user = create_rw_signal(cx, None::<String>);

    let users = create_resource(
        cx,
        move || current_list_page.get(),
        move |_current_list_page| async move {
            let query = ListQuery {
                page: Some(current_list_page.get()),
                page_size: Some(page_size.get() as u32),
                order_by: None,
            };
            match authorized_api.get() {
                Some(api) => match api.user_list(query).await {
                    Ok(users) => users,
                    Err(e) => {
                        let e = e.to_string();
                        goto_login_if_expired(cx, &e, authorized_api);
                        users_message.update(|msg| *msg = Message::Error(e));
                        vec![]
                    }
                },
                None => {
                    users_message
                        .update(|msg| *msg = Message::Error("connection to API failed".into()));
                    vec![]
                }
            }
        },
    );
    let elem_count = Signal::derive(cx, move || {
        users.read(cx).map(|users| users.len()).unwrap_or_default()
    });

    let remove_user_action = create_action(cx, move |username: &String| {
        let username = username.clone();
        async move {
            if let Some(api) = authorized_api.get() {
                let response = api.user_remove(&username).await;

                match response {
                    Ok(_) => {
                        users_message.update(|m| {
                            *m = Message::Success(format!("Successfully removed user {username}"));
                        });
                        current_list_page.update(|p| *p = *p);
                    }
                    Err(err) => users_message.update(|m| {
                        *m = Message::Error(format!("failed to remove user {username} - {err}"));
                    }),
                }
            } else {
                users_message.update(|m| {
                    *m = Message::Error("failed to connect to API".into());
                });
            }
        }
    });

    let dispatch_remove_user =
        move || remove_user_action.dispatch(remove_user.get().unwrap_or_default());

    let remove_confirm_modal = move || {
        view! { cx,
          <RemoveModal
            modal_id="removeUserModal"
            target="user"
            entry=remove_user.read_only()
            remove_action_fn=dispatch_remove_user
          />
        }
        .into_view(cx)
    };

    view! { cx,
      {move || {
            view!{ cx,
               <main class="bg-dark text-white d-flex flex-column p-1 pt-3" >
                 <div class="d-flex pb-3">
                     <h1 class="display-5 p-1">{Page::Users.title()}</h1>
                 </div>
                 <div class="btn-toolbar mx-3">
                     <a href="/users/add">
                         <button class="btn btn-outline-lighter rounded">
                             <img class="me-2" src="/icons/user-plus.svg" />
                             "Add"
                         </button>
                     </a>
                 </div>
                 <StatusMessage message=users_message/>
                 <div>
                 <Suspense fallback=move || view! {cx, <p> "Loading users..."</p> }>
                 { move || {
                    let users = users.read(cx);
                    {match users {
                      Some(users) => {
                          view!{cx,
                        <div class="card bg-darker m-3">
                            <div class="card-body">
                                <table class="table table-hover table-striped table-responsive text-white">
                                  <thead>
                                  <tr class="align-middle">
                                    <th scope="col">"Username"</th>
                                    <th scope="col">"Email"</th>
                                    <th scope="col">"Account type"</th>
                                    <th scope="col">"Registration date"</th>
                                    <th scope="col" class="col-2"></th>
                                  </tr>
                                  </thead>
                                  <tbody>
                                  {
                                  users.into_iter().map(|user| {
                                      view!{cx, <UserListEntry user remove_user=remove_user.write_only()></UserListEntry>}
                                  }).collect::<Vec<_>>()
                                  }
                                  </tbody>
                                </table>
                                <ListPageControl current_list_page elem_count page_size=page_size.read_only() />
                            </div>
                        </div>
                                  }.into_view(cx)
                      }
                      None => view!{cx, <p>"Error"</p>}.into_view(cx)
                    }}
                   }}
                 </Suspense>
                 </div>
               </main>
               {remove_confirm_modal}
            }.into_view(cx)
     }}
    }
}
