use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use types::{EdgeError, EdgeResult, EdgeToken};
use unleash_types::client_features::ClientFeatures;

pub mod memory;
#[cfg(feature = "red")]
pub mod redis;
#[cfg(feature = "aws")]
pub mod s3;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    pub ready: bool,
    pub error: Option<EdgeError>,
    pub last_fetch: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachedData {
    pub status: Status,
    pub client_features: ClientFeatures,
}

impl Default for CachedData {
    fn default() -> Self {
        CachedData {
            status: Status::default(),
            client_features: ClientFeatures {
                version: 2,
                features: vec![],
                segments: None,
                query: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FullState {
    pub status: Status,
    pub data: DashMap<String, CachedData>,
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
    async fn read_raw_toggles(&self, env: &String) -> EdgeResult<Option<ClientFeatures>>;
}

#[async_trait]
pub trait StatusSink {
    async fn set_status(&mut self, status: Status) -> EdgeResult<()>;
}

#[async_trait]
pub trait StatusSource {
    async fn get_status(&self) -> EdgeResult<Option<Status>>;
}

pub trait InitRepository {
    fn init(&self) -> FullState;
}

#[async_trait]
pub trait StatusRepository: Default + StatusSink + StatusSource {}
#[async_trait]
pub trait ToggleRepository: Default + ToggleSink + ToggleSource {}

#[async_trait]
pub trait Repository:
    Default + StatusRepository + ToggleRepository + TokenStore + InitRepository
{
}
