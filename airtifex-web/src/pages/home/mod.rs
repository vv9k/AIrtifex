pub mod dashboard;

pub use dashboard::*;

use crate::api::AuthorizedApi;
use crate::components::status_message::*;
use crate::pages::PageStack;
use crate::Page;

use airtifex_core::user::AuthenticatedUser;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Home(
    cx: Scope,
    authorized_api: RwSignal<Option<AuthorizedApi>>,
    user_info: RwSignal<Option<AuthenticatedUser>>,
    page_stack: RwSignal<PageStack>,
    global_message: RwSignal<Message>,
) -> impl IntoView {
    page_stack.update(|p| p.push(Page::Home));
    let window_size = crate::web_util::WindowSize::signal(cx).expect("window size");

    view! { cx,
      {move || {
       let inner_view = match user_info.get() {
        Some(_info) => {
            let classes = move || if window_size.get().width < 992 {
                "text-center d-flex flex-column mx-3 w-100"
            } else {
                "text-center d-flex flex-column justify-content-center mx-3 w-100"
            };
            view!{ cx,
            <div class=classes>
                <StatusMessage message=global_message />
                <Dashboard authorized_api global_message />
            </div>
            }.into_view(cx)
        },
        None => view!{ cx,
            <div class="container d-flex h-100 justify-content-center">
                <div class="row align-items-center h-100">
                    <div class="card bg-darker">
                        <div style="padding-left: 0 !important;padding-right: 0 !important;" class="card-body py-5">
                            <div class="col-md-8 mx-auto align-items-center d-flex flex-column">
                                <h1 class="display-5 text-center font-monospace">"Welcome to "<span class="fw-bold"><span class="text-airtifex">"AI"</span>"rtifex"</span></h1>
                                <p class="pt-5 pb-3">"You are not logged in."</p>
                                <A href=Page::Login.path() class="btn btn-airtifex">
                                    "Go to login page"
                                </A>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }.into_view(cx)
       };

       view!{ cx,
         <main class="bg-dark text-white overflow-scroll" >
            { inner_view }
         </main>
       }.into_view(cx)
      }}
    }
}
