use serde::{de, Deserialize, Serialize};
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::v1::{Context as ClockContext, Timestamp};

#[derive(
    Copy, Clone, Default, Debug, sqlx::FromRow, PartialEq, PartialOrd, Hash, sqlx::Type, Ord, Eq,
)]
#[sqlx(transparent)]
pub struct Uuid(sqlx::types::Uuid);

pub type V1Context = uuid::v1::Context;

#[derive(Debug, Error)]
pub enum UuidError {
    #[error("Failed to get system time - {0}")]
    InvalidSystemTime(#[from] std::time::SystemTimeError),
    #[error("Failed to generate UUID - {0}")]
    GenerationError(#[from] uuid::Error),
    #[error("Failed to deserialize UUID from slice - {0}")]
    Deserialization(#[from] sqlx::types::uuid::Error),
}

impl Uuid {
    pub fn generate_v1(context: &ClockContext, base_id: &Uuid) -> Result<Self, UuidError> {
        let mut buf = [b'!'; 36];
        base_id.uuid().simple().encode_lower(&mut buf);

        let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let secs = duration.as_secs();
        let subsec_nanos = duration.subsec_nanos();

        let timestamp = Timestamp::from_unix(&context, secs, subsec_nanos);
        Ok(Self::from_bytes(
            *uuid::Uuid::new_v1(timestamp, &buf[..6])?.as_bytes(),
        ))
    }

    pub fn new_v4() -> Self {
        Self::from_bytes(*uuid::Uuid::new_v4().as_bytes())
    }

    pub fn to_sqlx_type(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(*self.0.as_bytes())
    }

    pub fn inner_ref(&self) -> &sqlx::types::Uuid {
        &self.0
    }

    pub fn uuid(&self) -> &sqlx::types::Uuid {
        &self.0
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(sqlx::types::Uuid::from_bytes(bytes))
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, UuidError> {
        Ok(Self(sqlx::types::Uuid::from_slice(bytes)?))
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let uuid: String = Deserialize::deserialize(deserializer)?;
        uuid.parse().map_err(de::Error::custom)
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl FromStr for Uuid {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(*s.parse::<uuid::Uuid>()?.as_bytes()))
    }
}
