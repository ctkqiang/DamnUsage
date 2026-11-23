use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::ports::{EventPublisherPort, SecretVaultPort, WriteRepositoryPort};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveProviderCommand {
    pub provider_id: String,
}

pub struct RemoveProviderHandler {
    repository: Arc<dyn WriteRepositoryPort>,
    vault: Arc<dyn SecretVaultPort>,
    projector: Arc<UsageProjector>,
    publisher: Arc<dyn EventPublisherPort>,
}

impl RemoveProviderHandler {
    pub fn new(
        repository: Arc<dyn WriteRepositoryPort>,
        vault: Arc<dyn SecretVaultPort>,
        projector: Arc<UsageProjector>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            repository,
            vault,
            projector,
            publisher,
        }
    }

    pub async fn handle(&self, cmd: RemoveProviderCommand) -> DomainResult<()> {
        let existing = self.repository.get_account(&cmd.provider_id).await?;
        if existing.is_none() {
            return Err(DomainError::ProviderNotFound(cmd.provider_id));
        }

        let _ = self.vault.delete_credentials(&cmd.provider_id).await;
        self.repository.delete_account(&cmd.provider_id).await?;

        let envelope = DomainEventEnvelope::wrap(DomainEvent::ProviderRemoved {
            provider_id: cmd.provider_id,
        });
        self.repository.append_event(envelope.clone()).await?;
        self.projector.apply_event(&envelope).await;
        self.publisher.publish(envelope).await;

        Ok(())
    }
}
