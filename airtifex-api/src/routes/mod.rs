pub mod api;
pub mod r#static;

use crate::ToAxumResponse;
use airtifex_core::api_response::ApiResponse;

use axum::response::Response;
use serde::Serialize;

fn handle_db_result_as_json<T: Serialize>(result: crate::Result<T>) -> Response {
    match result {
        Ok(data) => ApiResponse::success(&data).ok(),
        Err(e) => ApiResponse::failure(e).internal_server_error(),
    }
}
