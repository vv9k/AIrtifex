pub mod chat;
pub mod home;
pub mod image;
pub mod login;
pub mod prompt;
pub mod users;

pub use self::{chat::*, home::*, image::*, login::*, prompt::*, users::*};

use crate::components::navbar::NavElement;

use gloo_storage::{LocalStorage, Storage};
use leptos::*;
use leptos_router::{use_navigate, NavigationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum Page {
    #[default]
    Home,
    Users,
    UserAdd,
    UserEdit(String),
    UserPasswordChange(String),
    UserProfile,
    Chat,
    ChatView(String),
    Prompt,
    PromptGenerate,
    PromptList,
    PromptView(String),
    GenerateImage,
    GeneratedImageView(String),
    Login,
}

impl Page {
    pub fn root_page(&self) -> Self {
        match self {
            Self::Users
            | Self::UserAdd
            | Self::UserEdit(_)
            | Self::UserPasswordChange(_)
            | Self::UserProfile => Self::Users,
            Self::Chat | Self::ChatView(_) => Self::Chat,
            Self::GenerateImage | Self::GeneratedImageView(_) => Self::GenerateImage,
            Self::Prompt | Self::PromptGenerate | Self::PromptList | Self::PromptView(_) => {
                Self::Prompt
            }
            Self::Home => Self::Home,
            Self::Login => Self::Login,
        }
    }
    pub fn raw_path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Users => "/users",
            Self::UserAdd => "/users/add",
            Self::UserEdit(_) => "/users/:username/edit",
            Self::UserPasswordChange(_) => "/users/:username/password",
            Self::UserProfile => "/users/profile",
            Self::Chat => "/chat",
            Self::ChatView(_) => "/chat/:chat_id",
            Self::Prompt | Self::PromptGenerate => "/prompt",
            Self::PromptList => "/prompt/history",
            Self::PromptView(_) => "/prompt/:prompt_id",
            Self::GenerateImage => "/gen/image",
            Self::GeneratedImageView(_) => "/gen/image/:image_id",
            Self::Login => "/login",
        }
    }

    pub fn path(&self) -> String {
        match self {
            Self::Home => "/".into(),
            Self::Users => "/users".into(),
            Self::UserAdd => "/users/add".into(),
            Self::UserEdit(u) => format!("/users/{u}/edit"),
            Self::UserPasswordChange(u) => format!("/users/{u}/password"),
            Self::UserProfile => "/users/profile".into(),
            Self::Chat => "/chat".into(),
            Self::ChatView(c) => format!("/chat/{c}"),
            Self::Prompt | Self::PromptGenerate => "/prompt".into(),
            Self::PromptList => "/prompt/history".into(),
            Self::PromptView(p) => format!("/prompt/{p}"),
            Self::GenerateImage => "/gen/image".into(),
            Self::GeneratedImageView(i) => format!("/gen/image/{i}"),
            Self::Login => "/login".into(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Home => "/icons/home.svg",
            Self::Users
            | Self::UserAdd
            | Self::UserEdit(_)
            | Self::UserPasswordChange(_)
            | Self::UserProfile => "/icons/users.svg",
            Self::Chat | Self::ChatView(_) => "/icons/message-circle.svg",
            Self::Prompt | Self::PromptGenerate | Self::PromptList | Self::PromptView(_) => {
                "/icons/terminal.svg"
            }
            Self::Login => "/icons/login.svg",
            Self::GenerateImage | Self::GeneratedImageView(_) => "/icons/image.svg",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Users
            | Self::UserAdd
            | Self::UserEdit(_)
            | Self::UserPasswordChange(_)
            | Self::UserProfile => "Users",
            Self::Chat | Self::ChatView(_) => "Chat",
            Self::Prompt => "Prompt",
            Self::PromptGenerate => "Generate Prompt",
            Self::PromptList => "Prompt History",
            Self::PromptView(_) => "Prompt",
            Self::Login => "Login",
            Self::GenerateImage | Self::GeneratedImageView(_) => "Generate Image",
        }
    }

    pub fn nav_display(&self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Users
            | Self::UserAdd
            | Self::UserEdit(_)
            | Self::UserPasswordChange(_)
            | Self::UserProfile => "Users",
            Self::Chat | Self::ChatView(_) => "Chat",
            Self::Prompt | Self::PromptView(_) => "Prompt",
            Self::PromptGenerate => "generate",
            Self::PromptList => "history",
            Self::Login => "Login",
            Self::GenerateImage | Self::GeneratedImageView(_) => "Generate Image",
        }
    }

    pub fn main_user_pages() -> &'static [NavElement] {
        &[
            NavElement::Main(Self::Home),
            NavElement::Main(Self::Chat),
            NavElement::Sub(Self::Prompt, &[Self::PromptGenerate, Self::PromptList]),
            NavElement::Main(Self::GenerateImage),
        ]
    }

    pub fn main_admin_pages() -> &'static [NavElement] {
        &[
            NavElement::Main(Self::Home),
            NavElement::Main(Self::Users),
            NavElement::Main(Self::Chat),
            NavElement::Sub(Self::Prompt, &[Self::PromptGenerate, Self::PromptList]),
            NavElement::Main(Self::GenerateImage),
        ]
    }
}

impl AsRef<str> for Page {
    fn as_ref(&self) -> &str {
        self.raw_path()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageStack {
    stack: [Page; 2],
}
impl PageStack {
    const DEFAULT_KEY: &'static str = "AIRTIFEX_PAGE_STACK";

    pub fn new(stack: [Page; 2]) -> Self {
        Self { stack }
    }

    pub fn load() -> Self {
        LocalStorage::get(Self::DEFAULT_KEY)
            .ok()
            .unwrap_or_else(|| Self::new([Page::Home, Page::Home]))
    }

    pub fn push(&mut self, page: Page) {
        if page == self.stack[0] {
            return;
        }
        self.stack.swap(0, 1);
        self.stack[0] = page;
        if let Err(e) = LocalStorage::set(Self::DEFAULT_KEY, self).map_err(|e| e.to_string()) {
            log::error!("failed to save page stack: {e}");
        }
    }

    pub fn current(&self) -> &Page {
        &self.stack[0]
    }

    pub fn parent(&self) -> &Page {
        &self.stack[1]
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
        navigate(Page::Login.raw_path(), Default::default()).expect("login page");
    }
}

pub fn goto(cx: Scope, page: impl AsRef<str>) -> Result<(), NavigationError> {
    let navigate = use_navigate(cx);
    navigate(page.as_ref(), Default::default())
}
