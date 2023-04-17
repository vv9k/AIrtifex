use airtifex_core::user::AuthenticatedUser;

use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod api;
mod components;
mod inference;
mod pages;
mod web_util;

use components::{navbar::*, status_message::Message};
use pages::*;

const DEFAULT_API_URL: &str = "/api";
const API_TOKEN_STORAGE_KEY: &str = "api-token";

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    provide_meta_context(cx);
    // -- signals -- //

    let authorized_api = create_rw_signal(cx, None::<api::AuthorizedApi>);
    let user_info = create_rw_signal(cx, None::<AuthenticatedUser>);
    let logged_in = Signal::derive(cx, move || user_info.get().is_some());
    let page_stack = create_rw_signal(cx, PageStack::new([Page::Home, Page::Home]));

    let global_message = create_rw_signal(cx, Message::Empty);
    let users_message = create_rw_signal(cx, Message::Empty);
    let subtitle = create_rw_signal(cx, None::<String>);
    let title = Signal::derive(cx, move || {
        if let Some(subtitle) = subtitle.get() {
            format!("AIrtifex - {subtitle}")
        } else {
            format!("AIrtifex")
        }
    });

    // -- actions -- //

    let fetch_user_info = create_action(cx, move |_| async move {
        match authorized_api.get() {
            Some(api) => match api.me().await {
                Ok(info) => {
                    log::info!("{info:?}");
                    user_info.update(|i| *i = Some(info));
                }
                Err(err) => {
                    log::error!("Unable to fetch user info: {err}")
                }
            },
            None => {
                log::error!("Unable to fetch user info: not logged in")
            }
        }
    });

    // -- init API -- //

    let unauthorized_api = api::UnauthorizedApi::new(DEFAULT_API_URL);
    if let Ok(token) = LocalStorage::get(API_TOKEN_STORAGE_KEY) {
        let api = api::AuthorizedApi::new(DEFAULT_API_URL, token);
        authorized_api.update(|a| *a = Some(api));
        fetch_user_info.dispatch(());
    }

    log::debug!("User is logged in: {}", logged_in.get());

    create_effect(cx, move |_| {
        log::debug!("API authorization state changed");
        match authorized_api.get() {
            Some(api) => {
                log::debug!("API is now authorized: save token in LocalStorage");
                LocalStorage::set(API_TOKEN_STORAGE_KEY, api.token()).expect("LocalStorage::set");
            }
            None => {
                log::debug!("API is no longer authorized: delete token from LocalStorage");
                LocalStorage::delete(API_TOKEN_STORAGE_KEY);
            }
        }
    });

    let logout = create_action(cx, move |_| async move {
        authorized_api.update(|api: &mut Option<api::AuthorizedApi>| {
            *api = None;
        });
        user_info.update(|user: &mut Option<AuthenticatedUser>| {
            *user = None;
        })
    });

    let on_logout = move || {
        logout.dispatch(());
    };

    view! { cx,
          <Link rel="icon" sizes="16x16 32x32 96x96 180x180 256x256 512x512" href="/favicon.ico" />
          <Script src="/popper.min.js" />
          <Script src="/bootstrap.min.js" />
          <Title text=move || title.get() />
          <Router>
            <main>
              <Routes>
                <Route
                  path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Home".into()));
                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <Home user_info page_stack global_message />
                      }
                  }
                />

    //####################################################################################################

                <ProtectedRoute
                  path=Page::Users.path()
                  condition=move |_| {
                      user_info.get().map(|user| user.is_admin()).unwrap_or_default()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Users".into()));
                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <Users authorized_api page_stack users_message />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::UserAdd.path()
                  condition=move |_| {
                      user_info.get().map(|user| user.is_admin()).unwrap_or_default()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Add user".into()));
                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <UserAdd authorized_api page_stack users_message />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::UserPasswordChange.path()
                  condition=move |_| {
                      user_info.get().map(|user| user.is_admin()).unwrap_or_default()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Change password".into()));
                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <UserPasswordChange authorized_api page_stack users_message />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::UserProfile.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Profile".into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <UserProfile page_stack user_info=user_info.read_only() />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::UserEdit.path()
                  condition=move |_| {
                      let params = use_params::<EditParams>(cx);
                      let username = params.get().ok().and_then(|p| p.username).unwrap_or_default();
                      user_info.get().map(|user| {
                          user.is_admin() || user.username == username
                      }).unwrap_or_default()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Edit user".into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <UserEdit authorized_api page_stack users_message />
                      }
                  }
                />

    //####################################################################################################

                <ProtectedRoute
                  path=Page::Chat.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Chat".into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <Chat authorized_api page_stack />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::ChatView.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Chat".into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <ChatView authorized_api page_stack />
                      }
                  }
                />

    //####################################################################################################

                <ProtectedRoute
                  path=Page::Prompt.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some(Page::Prompt.title().into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <PromptView authorized_api page_stack />
                      }
                  }
                />

    //####################################################################################################

                <ProtectedRoute
                  path=Page::GenerateImage.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some(Page::GenerateImage.title().into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <GenerateImage authorized_api page_stack />
                      }
                  }
                />
                <ProtectedRoute
                  path=Page::GeneratedImageView.path()
                  condition=move |_| {
                      user_info.get().is_some()
                  }
                  redirect_path=Page::Home.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some(Page::GeneratedImageView.title().into()));

                      view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <ImageView authorized_api page_stack />
                      }
                  }
                />

    //####################################################################################################

                <Route
                  path=Page::Login.path()
                  view=move |cx| {
                      subtitle.update(|sub| *sub = Some("Login".into()));
                      view! { cx,
                        <Login
                          api = unauthorized_api
                          on_success = move |api| {
                              log::info!("Successfully logged in");
                              authorized_api.update(|v| *v = Some(api));
                              let navigate = use_navigate(cx);
                              navigate(Page::Home.path(), Default::default()).expect("Home route");
                              fetch_user_info.dispatch(());
                          } />
                      }
                  }
                />
                <Route
                    path="*"
                    view=move |cx| {
                        subtitle.update(|sub| *sub = Some("404".into()));
                        global_message.update(|m| *m = Message::Error("Oh my 404! The page you're looking for doesn't exist so I brought you back home ;)".into()));
                        view! { cx,
                        <NavBar page_stack user_info on_logout />
                        <Home user_info page_stack global_message/>
                    }
                    }
                />
              </Routes>
            </main>
          </Router>
        }
}
