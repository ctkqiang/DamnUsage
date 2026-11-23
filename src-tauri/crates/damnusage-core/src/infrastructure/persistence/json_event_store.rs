use crate::domain::entities::{ProviderAccount, UserPreferences};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::events::DomainEventEnvelope;
use crate::domain::ports::WriteRepositoryPort;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PersistedState {
    accounts: HashMap<String, ProviderAccount>,
    events: Vec<DomainEventEnvelope>,
    preferences: UserPreferences,
}

pub struct JsonEventStore {
    file_path: PathBuf,
    cache: RwLock<PersistedState>,
}

impl JsonEventStore {
    pub fn open(base_dir: PathBuf) -> DomainResult<Self> {
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| DomainError::StorageError(e.to_string()))?;
        let file_path = base_dir.join("damnusage_store.json");
        let state = if file_path.exists() {
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| DomainError::StorageError(e.to_string()))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            PersistedState::default()
        };
        Ok(Self {
            file_path,
            cache: RwLock::new(state),
        })
    }

    pub fn in_memory() -> Self {
        Self {
            file_path: PathBuf::new(),
            cache: RwLock::new(PersistedState::default()),
        }
    }

    fn flush_sync(&self, state: &PersistedState) -> DomainResult<()> {
        if self.file_path.as_os_str().is_empty() {
            return Ok(());
        }
        let serialized = serde_json::to_string_pretty(state)
            .map_err(|e| DomainError::StorageError(e.to_string()))?;
        let tmp_path = self.file_path.with_extension("json.tmp");
        std::fs::write(&tmp_path, serialized)
            .map_err(|e| DomainError::StorageError(e.to_string()))?;
        std::fs::rename(&tmp_path, &self.file_path)
            .map_err(|e| DomainError::StorageError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl WriteRepositoryPort for JsonEventStore {
    async fn save_account(&self, account: ProviderAccount) -> DomainResult<()> {
        let mut guard = self.cache.write().await;
        guard.accounts.insert(account.id.clone(), account);
        self.flush_sync(&guard)
    }

    async fn delete_account(&self, provider_id: &str) -> DomainResult<()> {
        let mut guard = self.cache.write().await;
        guard.accounts.remove(provider_id);
        self.flush_sync(&guard)
    }

    async fn list_accounts(&self) -> DomainResult<Vec<ProviderAccount>> {
        let guard = self.cache.read().await;
        Ok(guard.accounts.values().cloned().collect())
    }

    async fn get_account(&self, provider_id: &str) -> DomainResult<Option<ProviderAccount>> {
        let guard = self.cache.read().await;
        Ok(guard.accounts.get(provider_id).cloned())
    }

    async fn append_event(&self, event: DomainEventEnvelope) -> DomainResult<()> {
        let mut guard = self.cache.write().await;
        guard.events.push(event);
        if guard.events.len() > 800 {
            let excess = guard.events.len() - 800;
            guard.events.drain(0..excess);
        }
        self.flush_sync(&guard)
    }

    async fn load_events(&self) -> DomainResult<Vec<DomainEventEnvelope>> {
        let guard = self.cache.read().await;
        Ok(guard.events.clone())
    }

    async fn save_preferences(&self, preferences: UserPreferences) -> DomainResult<()> {
        let mut guard = self.cache.write().await;
        guard.preferences = preferences;
        self.flush_sync(&guard)
    }

    async fn load_preferences(&self) -> DomainResult<UserPreferences> {
        let guard = self.cache.read().await;
        Ok(guard.preferences.clone())
    }
}
