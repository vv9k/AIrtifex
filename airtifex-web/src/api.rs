use airtifex_core::image::{ImageModelListEntry, TextToImageResponse};
use airtifex_core::{api_response::ApiResponse, auth::Credentials};
use airtifex_core::{
    image::{ImageGenerateRequest, ImageInspect, ImageSampleInspect},
    llm::{
        ChatEntryListEntry, ChatListEntry, ChatResponseRequest, ChatStartRequest,
        ChatStartResponse, LlmListEntry, OneshotInferenceRequest,
    },
    query::{append_query, UrlQuery},
    user::{
        self, AuthenticatedUser, GetUserEntry, ListUserEntry, PasswordChangeRequest,
        UserEditRequest, UserRegisterRequest,
    },
    JsonWebToken,
};

use gloo_net::http::{Request, Response};
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Clone, Copy)]
pub struct UnauthorizedApi {
    url: &'static str,
}

#[derive(Clone)]
pub struct AuthorizedApi {
    url: &'static str,
    token: JsonWebToken,
}

impl UnauthorizedApi {
    pub const fn new(url: &'static str) -> Self {
        Self { url }
    }

    pub async fn login(&self, credentials: &Credentials) -> Result<AuthorizedApi> {
        let url = format!("{}/users/login", self.url);
        let response = Request::post(&url).json(credentials)?.send().await?;
        let token = into_json(response).await?;
        Ok(AuthorizedApi::new(self.url, token))
    }
}

impl AuthorizedApi {
    pub const fn new(url: &'static str, token: JsonWebToken) -> Self {
        Self { url, token }
    }
    fn auth_header_value(&self) -> String {
        format!("Bearer {}", self.token.token)
    }
    async fn send_json<T>(&self, req: Request) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.send(req).await?;
        into_json(response).await
    }
    async fn send(&self, req: Request) -> Result<Response> {
        let response = req
            .header("Authorization", &self.auth_header_value())
            .send()
            .await
            .map_err(Error::from);
        // log::info!("got response {response:?}");
        response
    }
    pub async fn me(&self) -> Result<AuthenticatedUser> {
        let url = format!("{}/users/me", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn user_info(&self, username: &str) -> Result<GetUserEntry> {
        let url = format!("{}/users/{}", self.url, username);
        self.send_json(Request::get(&url)).await
    }
    pub async fn user_list(&self, query: user::ListQuery) -> Result<Vec<ListUserEntry>> {
        let url = append_query(format!("{}/users", self.url), query.as_query());
        self.send_json(Request::get(&url)).await
    }
    pub async fn user_add(&self, request: UserRegisterRequest) -> Result<String> {
        let url = format!("{}/users", self.url);
        self.send_json(Request::post(&url).json(&request)?).await
    }
    pub async fn user_edit(&self, username: &str, request: UserEditRequest) -> Result<()> {
        let url = format!("{}/users/{}", self.url, username);
        self.send_json(Request::post(&url).json(&request)?).await
    }
    pub async fn user_remove(&self, username: &str) -> Result<()> {
        let url = format!("{}/users/{}", self.url, username);
        self.send_json(Request::delete(&url)).await
    }
    pub async fn user_change_password(
        &self,
        username: &str,
        request: PasswordChangeRequest,
    ) -> Result<()> {
        let url = format!("{}/users/{}/password", self.url, username);
        self.send_json(Request::post(&url).json(&request)?).await
    }
    pub async fn chat_get_response(
        &self,
        request: ChatResponseRequest,
        id: &str,
    ) -> Result<Response> {
        let url = format!("{}/llm/chat/{id}", self.url);
        self.send(Request::post(&url).json(&request)?).await
    }
    pub async fn oneshot_inference(&self, request: OneshotInferenceRequest) -> Result<Response> {
        let url = format!("{}/llm/inference", self.url);
        self.send(Request::post(&url).json(&request)?).await
    }
    pub async fn chat_start_new(&self, request: ChatStartRequest) -> Result<ChatStartResponse> {
        let url = format!("{}/llm/chat", self.url);
        self.send_json(Request::post(&url).json(&request)?).await
    }
    pub async fn chat_history(&self, id: &str) -> Result<Vec<ChatEntryListEntry>> {
        let url = format!("{}/llm/chat/{id}/history", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn chat(&self, id: &str) -> Result<ChatListEntry> {
        let url = format!("{}/llm/chat/{id}", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn chat_remove(&self, id: &str) -> Result<()> {
        let url = format!("{}/llm/chat/{id}", self.url);
        self.send_json(Request::delete(&url)).await
    }
    pub async fn chat_list(&self) -> Result<Vec<ChatListEntry>> {
        let url = format!("{}/llm/chat", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn image_list(&self) -> Result<Vec<ImageInspect>> {
        let url = format!("{}/image", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn image_delete(&self, id: &str) -> Result<()> {
        let url = format!("{}/image/{id}", self.url);
        self.send_json(Request::delete(&url)).await
    }
    pub async fn image_info(&self, id: &str) -> Result<ImageInspect> {
        let url = format!("{}/image/{id}", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn image_samples(&self, id: &str) -> Result<Vec<ImageSampleInspect>> {
        let url = format!("{}/image/{id}/samples", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn image_generate(
        &self,
        request: ImageGenerateRequest,
    ) -> Result<TextToImageResponse> {
        let url = format!("{}/image/generate", self.url);
        self.send_json(Request::post(&url).json(&request)?).await
    }
    pub async fn large_language_models(&self) -> Result<Vec<LlmListEntry>> {
        let url = format!("{}/llm/models", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub async fn image_models(&self) -> Result<Vec<ImageModelListEntry>> {
        let url = format!("{}/image/models", self.url);
        self.send_json(Request::get(&url)).await
    }
    pub fn token(&self) -> &JsonWebToken {
        &self.token
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Fetch(#[from] gloo_net::Error),
    #[error(transparent)]
    DeserializeError(#[from] serde_json::Error),
    #[error(transparent)]
    ApiResponseError(#[from] airtifex_core::api_response::ResponseError),
    #[error("{0}")]
    ApiError(String),
}

async fn into_json<T>(response: Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let json = response.json::<ApiResponse>().await?;
    // log::info!("got json {json:?}");
    json.into_result(|e| Error::ApiError(e))
}
