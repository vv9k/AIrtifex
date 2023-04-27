use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ChatResponseRequest {
    pub prompt: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ChatStartResponse {
    pub chat_id: String,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "sql", derive(sqlx::Type))]
pub enum ChatEntryType {
    #[default]
    User = 1,
    Bot = 2,
}

impl ChatEntryType {
    pub fn to_str(self) -> &'static str {
        match self {
            ChatEntryType::User => "user",
            ChatEntryType::Bot => "bot",
        }
    }
    pub fn parse_str(s: impl AsRef<str>) -> Option<Self> {
        match s.as_ref() {
            "user" => Some(ChatEntryType::User),
            "bot" => Some(ChatEntryType::Bot),
            _ => None,
        }
    }
}

impl AsRef<str> for ChatEntryType {
    fn as_ref(&self) -> &str {
        match self {
            ChatEntryType::User => "user",
            ChatEntryType::Bot => "bot",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatListEntry {
    pub id: String,
    pub username: String,
    pub title: String,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub model: String,
    pub settings: InferenceSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatEntryListEntry {
    pub id: String,
    pub chat_id: String,
    pub entry_type: ChatEntryType,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatStartRequest {
    pub title: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub settings: InferenceSettings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmListEntry {
    pub model_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct InferenceSettings {
    pub num_predict: Option<usize>,
    pub system_prompt: Option<String>,
    pub n_batch: Option<usize>,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub temp: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneshotInferenceRequest {
    pub prompt: String,
    pub model: String,
    pub num_predict: Option<usize>,
    pub n_batch: Option<usize>,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub temp: Option<f32>,
    #[serde(default = "default_play_back_tokens")]
    pub play_back_tokens: bool,
    pub save: bool,
}

fn default_play_back_tokens() -> bool {
    true
}

pub type ChatStreamResult = Result<String, String>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserChatCounters {
    pub chat_count: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PromptInspect {
    pub id: String,
    pub username: String,
    pub prompt: String,
    pub response: String,
    pub date: chrono::DateTime<chrono::Utc>,
    pub model: String,
    pub num_predict: Option<usize>,
    pub n_batch: Option<usize>,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub temp: Option<f32>,
}
