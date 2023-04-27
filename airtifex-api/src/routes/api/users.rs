use crate::{
    auth::{generate_jwt, Claims, JsonWebToken},
    errors::Error,
    models::user::User,
    routes::handle_db_result_as_json,
    SharedAppState, ToAxumResponse,
};
use airtifex_core::{
    api_response::ApiResponse,
    auth::Credentials,
    user::{
        GetUserEntry, ListQuery, ListUserEntry, PasswordChangeRequest, UserEditRequest,
        UserRegisterRequest,
    },
};

use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing, Json, Router,
};

pub fn router() -> Router<SharedAppState> {
    Router::new()
        .route("/", routing::get(list).post(register))
        .route("/me", routing::get(me))
        .route("/login", routing::post(auth))
        .route("/:user", routing::get(info).post(update).delete(remove))
        .route("/:user/password", routing::post(change_password))
}

async fn me(claims: Claims, state: State<SharedAppState>) -> Response {
    let db = &state.db;
    let user = with_user_guard!(claims, db);
    ApiResponse::success(user).ok()
}

async fn info(
    claims: Claims,
    state: State<SharedAppState>,
    Path(username): Path<String>,
) -> Response {
    let db = &state.db;
    with_user_guard!(claims, db);
    if username != claims.sub && claims.role != "admin" {
        return ApiResponse::failure("Unauthorized to access user data").unauthorized();
    }
    handle_db_result_as_json(
        User::get(db, &username)
            .await
            .map(|user| GetUserEntry {
                id: user.id.to_string(),
                username: user.username,
                email: user.email,
                registration_date: user.registration_date,
                account_type: user.account_type,
            })
            .map_err(Error::from),
    )
}

async fn list(claims: Claims, state: State<SharedAppState>, query: Query<ListQuery>) -> Response {
    let db = &state.db;
    with_admin_guard!(claims, db);

    handle_db_result_as_json(
        User::list(db, query.page, query.page_size, query.order_by)
            .await
            .map_err(Error::from)
            .map(|users| {
                users
                    .into_iter()
                    .map(|user| ListUserEntry {
                        id: user.id.to_string(),
                        username: user.username,
                        email: user.email,
                        account_type: user.account_type,
                        registration_date: user.registration_date,
                    })
                    .collect::<Vec<_>>()
            }),
    )
}

async fn register(
    claims: Claims,
    state: State<SharedAppState>,
    user: Json<UserRegisterRequest>,
) -> Response {
    let db = &state.db;
    with_admin_guard!(claims, db);
    let user: User = user.0.into();
    handle_db_result_as_json(user.create(db).await.map(|_| user.id).map_err(Error::from))
}

async fn auth(state: State<SharedAppState>, credentials: Json<Credentials>) -> Response {
    match User::authenticate(&state.db, credentials.0).await {
        Ok(user) => {
            let token = match generate_jwt(&user.username, user.account_type) {
                Ok(token) => token,
                Err(e) => return ApiResponse::failure(e).unauthorized(),
            };

            ApiResponse::success(JsonWebToken { token }).ok()
        }
        Err(e) => ApiResponse::failure(e).unauthorized(),
    }
}

async fn change_password(
    claims: Claims,
    state: State<SharedAppState>,
    Path(username): Path<String>,
    request: Json<PasswordChangeRequest>,
) -> Response {
    let db = &state.db;
    with_admin_guard!(claims, db);
    if request.new_password.is_empty() {
        return ApiResponse::failure("Password cannot be empty").bad_request();
    }
    handle_db_result_as_json(
        User::change_pasword_by_username(db, &username, request.new_password.clone())
            .await
            .map_err(Error::from),
    )
}

async fn remove(
    claims: Claims,
    state: State<SharedAppState>,
    Path(username): Path<String>,
) -> Response {
    let db = &state.db;
    with_admin_guard!(claims, db);
    handle_db_result_as_json(
        User::delete_by_name(db, &username)
            .await
            .map_err(Error::from),
    )
}

async fn update(
    claims: Claims,
    state: State<SharedAppState>,
    Path(username): Path<String>,
    Json(request): Json<UserEditRequest>,
) -> Response {
    let db = &state.db;
    with_admin_guard!(claims, db);
    handle_db_result_as_json(
        User::update_by_name(db, &username, request.email, request.account_type)
            .await
            .map_err(Error::from),
    )
}
