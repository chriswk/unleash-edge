use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use types::{EdgeError, EdgeResult};
use unleash_types::client_features::ClientFeatures;

pub mod memory;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ApiTokenType {
    Client,
    Admin,
    Frontend,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EdgeToken {
    token: String,
    environment: String,
    projects: Vec<String>,
    token_type: ApiTokenType,
    dynamic: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    pub ready: bool,
    pub error: Option<EdgeError>,
    pub last_fetch: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait TokenStore {
    async fn is_valid(&self, token: String) -> EdgeResult<bool>;
    async fn get_tokens(&self) -> EdgeResult<Vec<EdgeToken>>;
    async fn get_token(&self, token: String) -> EdgeResult<Option<EdgeToken>>;
    async fn add_token(&self, token: EdgeToken) -> EdgeResult<()>;
    async fn remove_token(&self, token: String) -> EdgeResult<()>;
}

#[async_trait]
pub trait ToggleSink {
    async fn save_toggles(&self, env: String, data: ClientFeatures) -> EdgeResult<()>;
}

#[async_trait]
pub trait ToggleSource {
    async fn read_raw_toggles(&self, env: String) -> EdgeResult<Option<ClientFeatures>>;
}

#[async_trait]
pub trait StatusSink {
    async fn set_status(&self, status: Status) -> EdgeResult<()>;
}

#[async_trait]
pub trait StatusSource {
    async fn get_status(&self) -> EdgeResult<Option<Status>>;
}

#[async_trait]
pub trait StatusRepository: Default + StatusSink + StatusSource {}
#[async_trait]
pub trait ToggleRepository: Default + ToggleSink + ToggleSource {}

#[async_trait]
pub trait Repository: Default + StatusRepository + ToggleRepository + TokenStore {}
