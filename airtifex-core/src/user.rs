use crate::query::UrlQuery;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserEditRequest {
    pub email: String,
    pub account_type: AccountType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
    #[serde(default)]
    pub account_type: AccountType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub account_type: AccountType,
    pub registration_date: chrono::DateTime<chrono::Utc>,
}

impl AuthenticatedUser {
    pub fn is_user(&self) -> bool {
        self.account_type == AccountType::User
    }
    pub fn is_admin(&self) -> bool {
        self.account_type == AccountType::Admin
    }
}
//impl From<UserRegisterRequest> for User {
//fn from(req: UserRegisterRequest) -> Self {
//User::new(req.username, req.password, req.email, req.account_type)
//}
//}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
pub enum AccountType {
    Admin = 1,
    #[default]
    User = 2,
    Service = 4,
}

impl AccountType {
    pub fn to_str(self) -> &'static str {
        match self {
            AccountType::Admin => "admin",
            AccountType::User => "user",
            AccountType::Service => "service",
        }
    }
    pub fn parse_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "admin" => Some(AccountType::Admin),
            "user" => Some(AccountType::User),
            "service" => Some(AccountType::Service),
            _ => None,
        }
    }
}

impl AsRef<str> for AccountType {
    fn as_ref(&self) -> &str {
        match self {
            AccountType::Admin => "admin",
            AccountType::User => "user",
            AccountType::Service => "service",
        }
    }
}

pub type GetUserEntry = ListUserEntry;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListUserEntry {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub account_type: AccountType,
    pub registration_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ListOrder {
    #[serde(rename = "username")]
    Username,
    #[serde(rename = "email")]
    Email,
    #[serde(rename = "account_type")]
    AccountType,
    #[serde(rename = "registration_date")]
    RegistrationDate,
}

impl AsRef<str> for ListOrder {
    fn as_ref(&self) -> &str {
        match self {
            Self::AccountType => "account_type",
            Self::Email => "email",
            Self::Username => "username",
            Self::RegistrationDate => "registration_date",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub order_by: Option<ListOrder>,
}

impl UrlQuery for ListQuery {
    fn as_query(&self) -> String {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if let Some(page) = self.page {
            serializer.append_pair("page", &page.to_string());
        }
        if let Some(page_size) = self.page_size {
            serializer.append_pair("page_size", &page_size.to_string());
        }
        if let Some(order_by) = self.order_by {
            serializer.append_pair("order_by", order_by.as_ref());
        }
        serializer.finish()
    }
}
