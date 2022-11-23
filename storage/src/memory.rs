use crate::{
    EdgeResult, EdgeToken, Repository, Status, StatusRepository, StatusSink, StatusSource,
    ToggleRepository, ToggleSink, ToggleSource, TokenStore,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use unleash_types::client_features::ClientFeatures;

pub struct InMemoryRepository {
    toggle_store: Arc<RwLock<HashMap<String, ClientFeatures>>>,
    current_status: Option<Status>,
    token_store: Arc<RwLock<HashMap<String, EdgeToken>>>,
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        InMemoryRepository {
            toggle_store: Arc::new(RwLock::new(HashMap::new())),
            current_status: None,
            token_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ToggleSink for InMemoryRepository {
    async fn save_toggles(&mut self, env: String, data: ClientFeatures) -> EdgeResult<()> {
        self.toggle_store.insert(env, data);
        Ok(())
    }
}

impl ToggleSource for InMemoryRepository {
    async fn read_raw_toggles(&self, env: String) -> EdgeResult<Option<ClientFeatures>> {
        Ok(self.toggle_store.get(&env).map(|features| features.clone()))
    }
}

impl StatusRepository for InMemoryRepository {}

impl StatusSink for InMemoryRepository {
    async fn set_status(&mut self, status: Status) -> EdgeResult<()> {
        self.current_status = Some(status);
        Ok(())
    }
}

impl StatusSource for InMemoryRepository {
    async fn get_status(&self) -> EdgeResult<Option<Status>> {
        Ok(self.current_status.clone())
    }
}

impl TokenStore for InMemoryRepository {
    async fn is_valid(&self, token: String) -> EdgeResult<bool> {
        Ok(self.token_store.contains_key(&token))
    }

    async fn get_tokens(&self) -> EdgeResult<Vec<EdgeToken>> {
        Ok(self
            .token_store
            .values()
            .map(|token| token.clone())
            .collect())
    }

    async fn get_token(&self, token: String) -> EdgeResult<Option<EdgeToken>> {
        Ok(self.token_store.get(&token).map(|t| t.clone()))
    }

    async fn add_token(&mut self, token: EdgeToken) -> EdgeResult<()> {
        self.token_store.insert(token.clone().token, token.clone());
        Ok(())
    }

    async fn remove_token(&mut self, token: String) -> EdgeResult<()> {
        self.token_store.remove(&token);
        Ok(())
    }
}

impl ToggleRepository for InMemoryRepository {}

impl Repository for InMemoryRepository {}
