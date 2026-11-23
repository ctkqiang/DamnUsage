use crate::domain::entities::{ProviderAccount, UsageSnapshot, UserPreferences};
use crate::domain::errors::DomainResult;
use crate::domain::events::DomainEventEnvelope;
use crate::domain::value_objects::{NotchDisplayState, ProviderCredentials, ProviderKind};
use async_trait::async_trait;

#[async_trait]
pub trait UsageProviderPort: Send + Sync {
    fn kind(&self) -> ProviderKind;

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot>;
}

#[async_trait]
pub trait SecretVaultPort: Send + Sync {
    async fn save_credentials(
        &self,
        provider_id: &str,
        credentials: &ProviderCredentials,
    ) -> DomainResult<()>;

    async fn load_credentials(&self, provider_id: &str) -> DomainResult<ProviderCredentials>;

    async fn delete_credentials(&self, provider_id: &str) -> DomainResult<()>;
}

#[async_trait]
pub trait WriteRepositoryPort: Send + Sync {
    async fn save_account(&self, account: ProviderAccount) -> DomainResult<()>;
    async fn delete_account(&self, provider_id: &str) -> DomainResult<()>;
    async fn list_accounts(&self) -> DomainResult<Vec<ProviderAccount>>;
    async fn get_account(&self, provider_id: &str) -> DomainResult<Option<ProviderAccount>>;
    async fn append_event(&self, event: DomainEventEnvelope) -> DomainResult<()>;
    async fn load_events(&self) -> DomainResult<Vec<DomainEventEnvelope>>;
    async fn save_preferences(&self, preferences: UserPreferences) -> DomainResult<()>;
    async fn load_preferences(&self) -> DomainResult<UserPreferences>;
}

#[async_trait]
pub trait EventPublisherPort: Send + Sync {
    async fn publish(&self, event: DomainEventEnvelope);
}

#[async_trait]
pub trait NotchWindowPort: Send + Sync {
    async fn apply_state(&self, state: NotchDisplayState) -> DomainResult<()>;
}
