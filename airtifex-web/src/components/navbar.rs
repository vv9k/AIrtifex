use crate::{pages, Page, PageStack};
use airtifex_core::user::AuthenticatedUser;

use leptos::*;
use leptos_router::*;

pub enum NavElement {
    Main(Page),
    Sub(Page, &'static [Page]),
}

#[component]
pub fn NavItem(
    cx: Scope,
    page_stack: ReadSignal<PageStack>,
    nav: &'static NavElement,
) -> impl IntoView {
    match nav {
        NavElement::Main(page) => {
            let classes = move || {
                if page_stack.get().current().root_page() == *page {
                    "nav-link selected"
                } else {
                    "nav-link"
                }
            };
            view! { cx,
                <li class="nav-item sb-item">
                    <a href=page.raw_path() class=classes aria-current="page">
                        <img class="me-2" src=page.icon()/>
                        <span class="fw-bold text-white">{page.nav_display()}</span>
                    </a>
                </li>
            }
            .into_view(cx)
        }
        NavElement::Sub(root, sub) => {
            let is_current = move || page_stack.get().current().root_page() == *root;
            let aria_expanded = move || is_current().to_string();
            let collapsed = move || {
                if is_current() {
                    "collapse show"
                } else {
                    "collapse"
                }
            };
            let parent_classes = move || {
                if is_current() {
                    "btn btn-toggle text-start nav-link w-100 text-white fw-bold selected"
                } else {
                    "btn btn-toggle text-start nav-link w-100 collapsed text-white fw-bold"
                }
            };
            view! { cx,
                <li class="nav-item sb-item">
                  <button class=parent_classes data-bs-toggle="collapse" data-bs-target="#images-collapse" aria-expanded={aria_expanded}>
                      <img src=root.icon()/>
                      {root.nav_display()}
                  </button>
                  <div id="images-collapse" class=collapsed>
                    <ul class="list-unstyled fw-normal pb-1">
                      { move || {
                          sub.into_iter().map(|p| {
                            let classes = move || if page_stack.get().current() == p {
                              "ms-5 ps-2 nav-link selected"
                            } else {
                              "ms-5 ps-2 nav-link"
                            };
                            view!{ cx, 
                              <li 
                                class=classes
                                style="cursor: pointer;"
                                on:click=move |_| pages::goto(cx, p.raw_path()).expect("subpage")
                              >
                                <p class="text-white text-decoration-none fw-bold">"└ "{p.nav_display()}</p>
                              </li>
                            }.into_view(cx)
                          }).collect::<Vec<_>>()
                      }}
                    </ul>
                  </div>
                </li>
            }.into_view(cx)
        }
    }
}

#[component]
pub fn NavBar<F>(
    cx: Scope,
    page_stack: ReadSignal<PageStack>,
    user_info: RwSignal<Option<AuthenticatedUser>>,
    on_logout: F,
) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    view! { cx,
         {move || match user_info.get() {
         Some(user) => {
             let nav_items = if user.is_admin() { Page::main_admin_pages() } else { Page::main_user_pages() };
             let nav_items: Vec<_> = nav_items.into_iter().map(|nav| view!{cx, <NavItem page_stack nav/>}.into_view(cx)).collect();

             view!{cx,
       <nav class="sidebar d-flex flex-column flex-shrink-0 p-3 text-white bg-darker col-md-3 col-lg-2">
         <a href="/" class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-white text-decoration-none">
             <span class="fs-4 p-1 fw-bold font-monospace"><span style="color: #458588;">"AI"</span>"rtifex"</span>
         </a>
         <hr/>
         <ul class="nav nav-pills flex-column mb-auto">
         { nav_items }
         </ul>
         <hr/>
         <div class="dropdown">
           <a href="#" class="d-flex align-items-center text-white text-decoration-none dropdown-toggle" id="dropdownUser1" data-bs-toggle="dropdown" aria-expanded="false">
               <strong>{&user.username}</strong>
           </a>
           <ul class="dropdown-menu dropdown-menu-dark text-small shadow" aria-labelledby="dropdownUser1">
             <li><A class="dropdown-item" href=Page::UserProfile.raw_path()>"Profile"</A></li>
             <li><hr class="dropdown-divider"/></li>
             <li>
               <A
               href="#"
               class="dropdown-item"
               on:click={
                 let on_logout = on_logout.clone();
                 move |_| on_logout()
               }>
                   <img class="me-2" src="/icons/log-out.svg"/>
                   "Logout"
               </A>
             </li>
           </ul>
         </div>
       </nav>
      }.into_view(cx)
     }
     None => {
         view!{cx,
         <></>
         }.into_view(cx)
     }
    }}
         }
}
