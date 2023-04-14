use crate::auth::Claims;
use crate::models::user::{account_type_from_str, AuthenticationError, User};
use crate::permissions::Acl;
use crate::DbPool;
use airtifex_core::api_response::ApiResponse;
use airtifex_core::user::AuthenticatedUser;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[allow(unused_macros)]
macro_rules! with_optional_guard {
    ($req:ident, $db: ident) => {
        crate::guard::auth_guard(
            &$req,
            &$db,
            crate::permissions::Acl::builder().all().build(),
        )
        .await
        .ok()
    };
}

macro_rules! with_guard {
    ($req:ident, $db:ident, $acl:expr) => {
        match crate::guard::auth_guard(&$req, &$db, $acl).await {
            Ok(claims) => claims,
            Err(e) => return e,
        }
    };
    ($claims:ident, $db:ident, $redirect:expr, $acl:expr) => {
        if let Ok(authenticated_user) = crate::guard::auth_guard(&$claims, &$db, $acl).await {
            authenticated_user
        } else {
            return (
                axum::http::StatusCode::TEMPORARY_REDIRECT,
                [
                    ("Location", "/web"),
                    (
                        "Set-Cookie",
                        "Bearer=; Expires=Thu, Jan 01 1970 00:00:00 UTC;",
                    ),
                ],
            )
                .into_response();
        }
    };
}

macro_rules! with_admin_guard {
    ($req:ident, $db:ident) => {
        with_guard!($req, $db, crate::permissions::Acl::builder().build());
    };
    ($req:ident, $db:ident, $redirect: expr) => {
        with_guard!(
            $req,
            $db,
            $redirect,
            crate::permissions::Acl::builder().build()
        )
    };
}

#[allow(unused_macros)]
macro_rules! with_service_guard {
    ($req:ident, $db:ident) => {
        with_guard!(
            $req,
            $db,
            crate::permissions::Acl::builder().with_service().build()
        );
    };
    ($req:ident, $db:ident, $redirect: expr) => {
        with_guard!(
            $req,
            $db,
            $redirect,
            crate::permissions::Acl::builder().with_service().build()
        );
    };
}

macro_rules! with_user_guard {
    ($req:ident, $db:ident) => {
        with_guard!(
            $req,
            $db,
            crate::permissions::Acl::builder().with_user().build()
        )
    };
    ($req:ident, $db:ident, $redirect: expr) => {
        with_guard!(
            $req,
            $db,
            $redirect,
            crate::permissions::Acl::builder().with_user().build()
        )
    };
}

pub async fn auth_guard_err(
    claims: &Claims,
    db: &DbPool,
    acl: Acl,
) -> Result<AuthenticatedUser, crate::models::Error> {
    let account_type = account_type_from_str(&claims.role)?;

    if !acl.has_account_type(account_type) {
        return Err(AuthenticationError::Unauthorized.into());
    }

    match User::get(db, &claims.sub).await {
        Ok(user) => Ok(AuthenticatedUser {
            id: user.id.to_string(),
            username: user.username,
            account_type,
            registration_date: user.registration_date,
            email: user.email,
        }),
        Err(e) => Err(e),
    }
}

pub async fn auth_guard(
    claims: &Claims,
    db: &DbPool,
    acl: Acl,
) -> Result<AuthenticatedUser, Response> {
    auth_guard_err(claims, db, acl).await.map_err(|e| match e {
        crate::models::Error::AuthenticationError(e) => {
            (StatusCode::UNAUTHORIZED, Json(ApiResponse::failure(e))).into_response()
        }
        e => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::failure(e)),
        )
            .into_response(),
    })
}
