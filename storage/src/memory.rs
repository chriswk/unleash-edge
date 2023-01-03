use crate::{
    Repository, Status, StatusRepository, StatusSink, StatusSource, ToggleRepository, ToggleSink,
    ToggleSource, TokenStore,
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use types::{EdgeResult, EdgeToken};
use unleash_types::client_features::ClientFeatures;

#[derive(Clone)]
pub struct InMemoryRepository {
    toggle_store: Arc<DashMap<String, ClientFeatures>>,
    current_status: Option<Status>,
    token_store: Arc<DashMap<String, EdgeToken>>,
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        InMemoryRepository {
            toggle_store: Arc::new(DashMap::new()),
            current_status: None,
            token_store: Arc::new(DashMap::new()),
        }
    }
}
#[async_trait]
impl ToggleSink for InMemoryRepository {
    async fn save_toggles(&self, env: String, data: ClientFeatures) -> EdgeResult<()> {
        self.toggle_store.insert(env, data);
        Ok(())
    }
}
#[async_trait]
impl ToggleSource for InMemoryRepository {
    async fn read_raw_toggles(&self, env: &String) -> EdgeResult<Option<ClientFeatures>> {
        Ok(self.toggle_store.get(env).map(|features| features.clone()))
    }
}

impl StatusRepository for InMemoryRepository {}

#[async_trait]
impl StatusSink for InMemoryRepository {
    async fn set_status(&mut self, status: Status) -> EdgeResult<()> {
        self.current_status = Some(status);
        Ok(())
    }
}
#[async_trait]
impl StatusSource for InMemoryRepository {
    async fn get_status(&self) -> EdgeResult<Option<Status>> {
        Ok(self.current_status.clone())
    }
}
#[async_trait]
impl TokenStore for InMemoryRepository {
    async fn is_valid(&self, token: String) -> EdgeResult<bool> {
        Ok(self.token_store.contains_key(&token))
    }

    async fn get_tokens(&self) -> EdgeResult<Vec<EdgeToken>> {
        Ok(self
            .token_store
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn get_token(&self, token: String) -> EdgeResult<Option<EdgeToken>> {
        Ok(self.token_store.get(&token).map(|token| token.clone()))
    }

    async fn add_token(&self, token: EdgeToken) -> EdgeResult<()> {
        self.token_store.insert(token.clone().token, token.clone());
        Ok(())
    }

    async fn remove_token(&self, token: String) -> EdgeResult<()> {
        self.token_store.remove(&token);
        Ok(())
    }
}
#[async_trait]
impl ToggleRepository for InMemoryRepository {}

#[async_trait]
impl Repository for InMemoryRepository {}

#[cfg(test)]
mod tests {
    use super::*;
}
