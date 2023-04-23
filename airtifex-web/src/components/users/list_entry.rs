use airtifex_core::user::ListUserEntry;

use leptos::*;

#[component]
pub fn UserListEntry(
    cx: Scope,
    user: ListUserEntry,
    remove_user: WriteSignal<Option<String>>,
) -> impl IntoView {
    let pw_change_href = format!("/users/{}/password", &user.username);
    let edit_href = format!("/users/{}/edit", &user.username);
    let edit_href2 = edit_href.clone();
    view! {cx,
      <tr class="text-white no-border align-middle">
          <td
            style="cursor: pointer;"
            on:click = move |_| {
                crate::pages::goto(cx, &edit_href2).expect("user edit page");
            }
          >{ user.username.clone() }</td>
          <td>{ user.email }</td>
          <td>{ user.account_type.to_str() }</td>
          <td>{ user.registration_date.format("%a, %d %b %Y %H:%M:%S").to_string() }</td>
          <td>
              <div class="btn-group" role="user toolbar" aria-label="user toolbar">
                  <button
                    data-bs-toggle="modal"
                    data-bs-target="#removeUserModal"
                    on:focus=move |_| {
                        remove_user.update(|c| *c = Some(user.username.clone()))
                    }
                    class="btn btn-outline-lighter"
                  >
                      <img src="/icons/user-minus.svg" />
                      "Remove"
                  </button>
                  <a href=edit_href class="btn btn-outline-lighter">
                      <img src="/icons/edit.svg" />
                      "Edit"
                  </a>
                  <a href=pw_change_href class="btn btn-outline-lighter d-flex flex-row">
                      <img src="/icons/key.svg" />
                      "Change password"
                  </a>
              </div>
          </td>
      </tr>
    }
    .into_view(cx)
}
