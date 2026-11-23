use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::SecretVaultPort;
use crate::domain::value_objects::ProviderCredentials;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

pub struct FileSecretVault {
    file_path: PathBuf,
    secrets: RwLock<HashMap<String, ProviderCredentials>>,
}

impl FileSecretVault {
    pub fn open(base_dir: PathBuf) -> DomainResult<Self> {
        std::fs::create_dir_all(&base_dir)
            .map_err(|e| DomainError::VaultError(e.to_string()))?;
        let file_path = base_dir.join("damnusage_vault.json");
        let secrets = if file_path.exists() {
            let raw = std::fs::read_to_string(&file_path)
                .map_err(|e| DomainError::VaultError(e.to_string()))?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            HashMap::new()
        };
        Ok(Self {
            file_path,
            secrets: RwLock::new(secrets),
        })
    }

    pub fn in_memory() -> Self {
        Self {
            file_path: PathBuf::new(),
            secrets: RwLock::new(HashMap::new()),
        }
    }

    fn flush_sync(&self, map: &HashMap<String, ProviderCredentials>) -> DomainResult<()> {
        if self.file_path.as_os_str().is_empty() {
            return Ok(());
        }
        let encoded = serde_json::to_string_pretty(map)
            .map_err(|e| DomainError::VaultError(e.to_string()))?;
        std::fs::write(&self.file_path, encoded)
            .map_err(|e| DomainError::VaultError(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &self.file_path,
                std::fs::Permissions::from_mode(0o600),
            );
        }
        Ok(())
    }
}

#[async_trait]
impl SecretVaultPort for FileSecretVault {
    async fn save_credentials(
        &self,
        provider_id: &str,
        credentials: &ProviderCredentials,
    ) -> DomainResult<()> {
        let mut guard = self.secrets.write().await;
        guard.insert(provider_id.to_string(), credentials.clone());
        self.flush_sync(&guard)
    }

    async fn load_credentials(&self, provider_id: &str) -> DomainResult<ProviderCredentials> {
        let guard = self.secrets.read().await;
        Ok(guard.get(provider_id).cloned().unwrap_or_default())
    }

    async fn delete_credentials(&self, provider_id: &str) -> DomainResult<()> {
        let mut guard = self.secrets.write().await;
        guard.remove(provider_id);
        self.flush_sync(&guard)
    }
}
