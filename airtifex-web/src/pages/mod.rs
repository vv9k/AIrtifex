pub mod chat;
pub mod home;
pub mod login;
pub mod prompt;
pub mod users;

pub use self::{chat::*, home::*, login::*, prompt::*, users::*};

use leptos::*;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Page {
    #[default]
    Home,
    Users,
    UserAdd,
    UserEdit,
    UserPasswordChange,
    UserProfile,
    Chat,
    ChatView,
    Prompt,
    Login,
}

impl Page {
    pub fn root_page(&self) -> Self {
        match self {
            Self::Users
            | Self::UserAdd
            | Self::UserEdit
            | Self::UserPasswordChange
            | Self::UserProfile => Self::Users,
            Self::Chat | Self::ChatView => Self::Chat,
            Self::Prompt | Self::Home | Self::Login => *self,
        }
    }
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Users => "/users",
            Self::UserAdd => "/users/add",
            Self::UserEdit => "/users/:username/edit",
            Self::UserPasswordChange => "/users/:username/password",
            Self::UserProfile => "/users/profile",
            Self::Chat => "/chat",
            Self::ChatView => "/chat/:chat_id",
            Self::Prompt => "/prompt",
            Self::Login => "/login",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Home => "/icons/home.svg",
            Self::Users
            | Self::UserAdd
            | Self::UserEdit
            | Self::UserPasswordChange
            | Self::UserProfile => "/icons/users.svg",
            Self::Chat | Self::ChatView => "/icons/message-circle.svg",
            Self::Prompt => "/icons/terminal.svg",
            Self::Login => "/icons/login.svg",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Users
            | Self::UserAdd
            | Self::UserEdit
            | Self::UserPasswordChange
            | Self::UserProfile => "Users",
            Self::Chat | Self::ChatView => "Chat",
            Self::Prompt => "Prompt",
            Self::Login => "Login",
        }
    }

    pub fn main_user_pages() -> &'static [Self] {
        &[Self::Home, Self::Chat, Self::Prompt]
    }

    pub fn main_admin_pages() -> &'static [Self] {
        &[Self::Home, Self::Users, Self::Chat, Self::Prompt]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PageStack {
    stack: [Page; 2],
}
impl PageStack {
    pub fn new(stack: [Page; 2]) -> Self {
        Self { stack }
    }

    pub fn push(&mut self, page: Page) {
        self.stack[0] = self.stack[1];
        self.stack[1] = page;
    }

    pub fn current(&self) -> Page {
        self.stack[1]
    }

    pub fn parent(&self) -> Page {
        self.stack[0]
    }
}

pub fn goto_login_if_expired(
    cx: Scope,
    e: impl AsRef<str>,
    api: RwSignal<Option<crate::api::AuthorizedApi>>,
) {
    use leptos_router::*;

    if e.as_ref().contains("ExpiredSignature") {
        api.update(|a| *a = None);
        let navigate = use_navigate(cx);
        navigate(Page::Login.path(), Default::default()).expect("login page");
    }
}

#[wasm_bindgen]
pub fn sleep(ms: i32) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    })
}

pub fn wasm_sleep(ms: i32) -> wasm_bindgen_futures::JsFuture {
    wasm_bindgen_futures::JsFuture::from(sleep(ms))
}
