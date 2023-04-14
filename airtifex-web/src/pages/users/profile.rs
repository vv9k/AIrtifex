use crate::{Page, PageStack};
use airtifex_core::user::AuthenticatedUser;

use leptos::*;
use leptos_router::*;

#[component]
pub fn UserProfile(
    cx: Scope,
    page_stack: RwSignal<PageStack>,
    user_info: ReadSignal<Option<AuthenticatedUser>>,
) -> impl IntoView {
    view! { cx,
      {move || {
        match user_info.get() {
            Some(user) => {
                page_stack.update(|p| p.push(Page::UserProfile));
                view!{cx,
                   <main class="bg-dark text-white d-flex flex-column p-1 pt-3" >
                         <div class="d-flex pb-2">
                             <h1 class="display-5 p-1">"Profile"</h1>
                         </div>
                         <div class="btn-toolbar mx-3">
                             <a href=format!("/users/{}/edit", user.username)>
                                 <button class="btn btn-outline-lighter rounded">
                                     <img class="me-2" src="/icons/edit.svg" />
                                     "Edit"
                                 </button>
                             </a>
                         </div>
                         <div class="card bg-darker m-3">
                             <div class="card-body">
                                <table class="table table-hover text-white mb-0">
                                <tbody>
                                    <tr class="no-border">
                                        <td class="fitwidth text-white">"Registration date:"</td>
                                        <td class="text-airtifex">
                                          { user.registration_date.format("%a, %d %b %Y %H:%M:%S").to_string() }
                                        </td>
                                    </tr>
                                    <tr class="no-border">
                                        <td class="fitwidth text-white">"ID:"</td>
                                        <td class="text-airtifex">{user.id}</td>
                                    </tr>
                                    <tr class="no-border">
                                        <td class="fitwidth text-white">"Username:"</td>
                                        <td class="text-airtifex">{user.username}</td>
                                    </tr>
                                    <tr class="no-border">
                                        <td class="fitwidth text-white">"Account type:"</td>
                                        <td class="text-airtifex">{user.account_type.to_str()}</td>
                                    </tr>
                                    <tr class="no-border">
                                        <td class="fitwidth text-white">"Email:"</td>
                                        <td class="text-airtifex">{user.email}</td>
                                    </tr>
                                </tbody>
                                </table>
                                <div class="flex-fill"></div>
                             </div>
                         </div>
                   </main>
                }.into_view(cx)
            }
            None => {
                let navigate = use_navigate(cx);
                navigate(Page::Home.path(), Default::default()).expect("Home page");
                view!{cx, <></>}.into_view(cx)
            }
        }
     }}
    }
}
