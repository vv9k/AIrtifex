use crate::{Page, PageStack};
use airtifex_core::user::AuthenticatedUser;

use leptos::*;
use leptos_router::*;

#[component]
pub fn NavItem(cx: Scope, page_stack: RwSignal<PageStack>, page: Page) -> impl IntoView {
    view! { cx,
        <li class="nav-item sb-item">
          <Show
            when = move || page_stack.get().current().root_page() == page
            fallback = move |cx| view! { cx,
               <a href=page.path() class="nav-link" aria-current="page">
                   <img class="me-2" src=page.icon()/>
                   <span class="fw-bold text-white">{page.title()}</span>
               </a>
            }
          >
             <a href=page.path() class="nav-link selected" aria-current="page">
                 <img class="me-2" src=page.icon()/>
                 <span class="fw-bold text-white">{page.title()}</span>
             </a>
          </Show>
        </li>
    }
}

#[component]
pub fn NavBar<F>(
    cx: Scope,
    page_stack: RwSignal<PageStack>,
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
             let nav_items: Vec<_> = nav_items.into_iter().map(|&page| view!{cx, <NavItem page_stack page/>}.into_view(cx)).collect();

             view!{cx,
       <nav class="sidebar d-flex flex-column flex-shrink-0 p-3 text-white bg-darker col-md-3 col-lg-2">
         <a href="/" class="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-white text-decoration-none">
             <span class="fs-4 p-1 fw-bold font-monospace"><span style="color: #458588;">"AI"</span>"rtifex"</span>
         </a>
         <hr/>
         <ul class="nav nav-pills flex-column mb-auto">
         { nav_items }
            // <li class="nav-item sb-item">
            //   <button class="btn btn-toggle align-items-center collapsed text-white fw-bold" data-bs-toggle="collapse" data-bs-target="#images-collapse" aria-expanded="false">
            //       <img src=""/>
            //       "Stable Diffusion"
            //   </button>
            //   <div id="images-collapse" class="collapse">
            //     <ul class="list-unstyled fw-normal pb-1">
            //       <li class="ms-5 ps-2 nav-link">
            //         <a href="" class="text-white text-decoration-none fw-bold">"└ Text to image"</a>
            //       </li>
            //     </ul>
            //   </div>
            // </li>
         </ul>
         <hr/>
         <div class="dropdown">
           <a href="#" class="d-flex align-items-center text-white text-decoration-none dropdown-toggle" id="dropdownUser1" data-bs-toggle="dropdown" aria-expanded="false">
               <strong>{&user.username}</strong>
           </a>
           <ul class="dropdown-menu dropdown-menu-dark text-small shadow" aria-labelledby="dropdownUser1">
             <li><A class="dropdown-item" href=Page::UserProfile.path()>"Profile"</A></li>
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
