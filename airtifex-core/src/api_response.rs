use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Failed to deserialize response data as `{0}` - {1}")]
    DataDeserializationError(&'static str, serde_json::Error),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiVersion {
    V1,
}

impl AsRef<str> for ApiVersion {
    fn as_ref(&self) -> &str {
        match self {
            ApiVersion::V1 => "v1",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    status: ResponseStatus,
    api_version: ApiVersion,
    timestamp: DateTime<Utc>,
    data: serde_json::Value,
}

impl ApiResponse {
    pub fn success<T: Serialize>(data: T) -> Self {
        match serde_json::to_value(&data) {
            Ok(data) => Self {
                status: ResponseStatus::Success,
                api_version: ApiVersion::V1,
                timestamp: Utc::now(),
                data,
            },
            Err(e) => Self::failure(e),
        }
    }

    pub fn failure<E: std::fmt::Display>(error: E) -> Self {
        log::error!("failure! {error}");
        Self {
            status: ResponseStatus::Failure,
            api_version: ApiVersion::V1,
            timestamp: Utc::now(),
            data: serde_json::Value::String(format!("{}", error)),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self.status, ResponseStatus::Success)
    }

    pub fn into_data(self) -> serde_json::Value {
        self.data
    }

    pub fn into_result<T: DeserializeOwned, E: From<ResponseError>>(
        self,
        f: impl FnOnce(String) -> E,
    ) -> Result<T, E> {
        if self.is_success() {
            self.deserialize_as().map_err(E::from)
        } else {
            Err(f(self.into_data().as_str().unwrap().to_string()))
        }
    }

    pub fn deserialize_as<T: DeserializeOwned>(self) -> Result<T, ResponseError> {
        serde_json::from_value(self.data)
            .map_err(|e| ResponseError::DataDeserializationError(std::any::type_name::<T>(), e))
    }
}
