use crate::{
    id::Uuid,
    models::{Error, Result},
};
use airtifex_core::{
    auth::{hash_pass, Credentials},
    user::{AccountType, ListOrder, UserRegisterRequest},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error as ErrorType;

use crate::DbPool;
use sqlx::FromRow;

#[derive(Debug, ErrorType)]
pub enum UserError {
    #[error("Failed to update password - {0}")]
    PasswordUpdateError(sqlx::Error),
    #[error("Failed to update a user - {0}")]
    UpdateError(sqlx::Error),
    #[error("Failed to delete a user - {0}")]
    DeleteError(sqlx::Error),
    #[error("Failed to create a new user - {0}")]
    CreateError(sqlx::Error),
    #[error("Failed to list users - {0}")]
    ListError(sqlx::Error),
    #[error("Invalid account type `{0}`")]
    InvalidAccountType(String),
}

#[derive(Debug, ErrorType)]
pub enum AuthenticationError {
    #[error("Failed to authenticate - invalid username or password")]
    AuthenticationFailed,
    #[error("The authenticated user has insufficient permissions")]
    Unauthorized,
    #[error("The authenticated user doesn't exist")]
    UserDoesntExist,
}

impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        let (user, pass) = credentials.consume();
        User::new(user, pass, "", AccountType::default())
    }
}

pub fn account_type_from_str(s: &str) -> core::result::Result<AccountType, UserError> {
    match &s.to_lowercase()[..] {
        "admin" => Ok(AccountType::Admin),
        "user" => Ok(AccountType::User),
        "service" => Ok(AccountType::Service),
        s => Err(UserError::InvalidAccountType(s.to_string())),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub password: Vec<u8>,
    #[serde(skip_deserializing)]
    pub email: String,
    pub account_type: AccountType,
    pub registration_date: DateTime<Utc>,
}

impl User {
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        email: impl Into<String>,
        account_type: AccountType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.into(),
            password: hash_pass(password.into()),
            email: email.into(),
            account_type,
            registration_date: Utc::now(),
        }
    }

    pub fn admin(mut self) -> Self {
        self.account_type = AccountType::Admin;
        self
    }

    pub fn service(mut self) -> Self {
        self.account_type = AccountType::Service;
        self
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.account_type, AccountType::Admin)
    }
    pub fn is_user(&self) -> bool {
        matches!(self.account_type, AccountType::User)
    }
    pub fn is_service(&self) -> bool {
        matches!(self.account_type, AccountType::Service)
    }
}

impl User {
    pub async fn create(&self, db: &DbPool) -> Result<()> {
        sqlx::query(
            r#"
           INSERT INTO users (id, username, email, password, account_type, registration_date)
           VALUES ($1, $2, $3, $4, $5, $6)
           "#,
        )
        .bind(self.id)
        .bind(&self.username)
        .bind(&self.email)
        .bind(&self.password)
        .bind(self.account_type)
        .bind(self.registration_date)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::CreateError)
        .map_err(Error::from)
    }

    /// Checks if the user with `username` exists and if so returns the ID
    pub async fn exists(db: &DbPool, username: &str) -> Option<String> {
        #[derive(Debug, FromRow)]
        struct UserId {
            id: Uuid,
        }

        sqlx::query_as(
            r#"
            SELECT id
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(db)
        .await
        .map(|id: UserId| id.id.to_string())
        .ok()
    }

    pub async fn list(
        db: &DbPool,
        page: Option<u32>,
        page_size: Option<u32>,
        order_by: Option<ListOrder>,
    ) -> Result<Vec<User>> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(25);
        let order_by = order_by.unwrap_or(ListOrder::AccountType);
        let order_by = match order_by {
            ListOrder::AccountType => "account_type",
            ListOrder::Email => "email",
            ListOrder::RegistrationDate => "registration_date",
            ListOrder::Username => "username",
        };
        let offset = (page - 1) * page_size;
        sqlx::query_as(
            r#"
            SELECT id, username, email, password, account_type, registration_date
            FROM users
            ORDER BY $1
            LIMIT $2
            OFFSET $3;
            "#,
        )
        .bind(order_by)
        .bind(page_size as i32)
        .bind(offset as i32)
        .fetch_all(db)
        .await
        .map_err(UserError::ListError)
        .map_err(Error::from)
    }

    pub async fn get(db: &DbPool, username: &str) -> Result<Self> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password, account_type, registration_date
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_one(db)
        .await
        .map_err(UserError::ListError)
        .map_err(Error::from)
    }

    pub async fn get_by_id(db: &DbPool, id: &Uuid) -> Result<Self> {
        sqlx::query_as(
            r#"
            SELECT id, username, email, password, account_type, registration_date
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(db)
        .await
        .map_err(UserError::ListError)
        .map_err(Error::from)
    }

    pub async fn update_by_name(
        db: &DbPool,
        username: &str,
        email: String,
        account_type: AccountType,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET email = $1, account_type = $2
            WHERE username = $3
            "#,
        )
        .bind(email)
        .bind(account_type)
        .bind(username)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::UpdateError)
        .map_err(Error::from)
    }

    pub async fn delete(db: &DbPool, id: &Uuid) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::DeleteError)
        .map_err(Error::from)
    }

    pub async fn delete_by_name(db: &DbPool, username: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::DeleteError)
        .map_err(Error::from)
    }

    pub async fn authenticate(db: &DbPool, credentials: Credentials) -> Result<Self> {
        let pass = credentials.password_digest();
        sqlx::query_as(
            r#"
            SELECT id, username, email, password, account_type, registration_date
            FROM users
            WHERE username = $1 AND password = $2
            "#,
        )
        .bind(credentials.username())
        .bind(pass)
        .fetch_one(db)
        .await
        .map_err(|_| AuthenticationError::AuthenticationFailed)
        .map_err(Error::from)
    }

    pub async fn change_pasword(db: &DbPool, user_id: &Uuid, new_password: String) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password = $1
            WHERE id = $2
            "#,
        )
        .bind(hash_pass(new_password))
        .bind(user_id)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::PasswordUpdateError)
        .map_err(Error::from)
    }

    pub async fn change_pasword_by_username(
        db: &DbPool,
        username: &str,
        new_password: String,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password = $1
            WHERE username = $2
            "#,
        )
        .bind(hash_pass(new_password))
        .bind(username)
        .execute(db)
        .await
        .map(|_| ())
        .map_err(UserError::PasswordUpdateError)
        .map_err(Error::from)
    }
}

impl From<UserRegisterRequest> for User {
    fn from(req: UserRegisterRequest) -> Self {
        User::new(req.username, req.password, req.email, req.account_type)
    }
}
