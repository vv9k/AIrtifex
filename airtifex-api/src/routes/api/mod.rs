pub mod chat;
pub mod image;
pub mod prompt;
pub mod users;

use crate::ApiVersion;

use axum::Router;

pub fn router() -> Router<crate::SharedAppState> {
    let base = Router::new()
        .nest("/users", users::router())
        .nest("/llm", chat::router().merge(prompt::router()))
        .nest("/image", image::router());

    Router::new().nest(&format!("/api/{}", ApiVersion::V1.as_ref()), base)
}
