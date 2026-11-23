use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::catalog::catalog_entry_for;
use crate::domain::entities::ProviderAccount;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::ports::{EventPublisherPort, SecretVaultPort, WriteRepositoryPort};
use crate::domain::value_objects::{ProviderCredentials, ProviderKind};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureProviderCommand {
    pub provider_id: Option<String>,
    pub kind: ProviderKind,
    pub label: String,
    pub enabled: bool,
    pub monthly_budget_usd: Option<f64>,
    pub endpoint_override: Option<String>,
    pub credentials: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

pub struct ConfigureProviderHandler {
    repository: Arc<dyn WriteRepositoryPort>,
    vault: Arc<dyn SecretVaultPort>,
    projector: Arc<UsageProjector>,
    publisher: Arc<dyn EventPublisherPort>,
}

impl ConfigureProviderHandler {
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

    pub async fn handle(&self, cmd: ConfigureProviderCommand) -> DomainResult<ProviderAccount> {
        let spec = catalog_entry_for(cmd.kind);
        let label = if cmd.label.trim().is_empty() {
            spec.display_name.clone()
        } else {
            cmd.label.trim().to_string()
        };

        let existing = match &cmd.provider_id {
            Some(id) => self.repository.get_account(id).await?,
            None => None,
        };

        let is_new = existing.is_none();
        if is_new {
            for field in &spec.credential_fields {
                if field.required {
                    let val = cmd
                        .credentials
                        .get(&field.key)
                        .map(|s| s.trim())
                        .unwrap_or("");
                    if val.is_empty() {
                        return Err(DomainError::MissingCredential(field.key.clone()));
                    }
                }
            }
        }

        let now = Utc::now();
        let id = cmd
            .provider_id
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("{}-{}", cmd.kind.as_str(), &Uuid::new_v4().to_string()[..8]));

        let account = ProviderAccount {
            id: id.clone(),
            kind: cmd.kind,
            label,
            enabled: cmd.enabled,
            monthly_budget_usd: cmd.monthly_budget_usd.filter(|b| *b > 0.0),
            endpoint_override: cmd
                .endpoint_override
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            metadata: cmd.metadata,
            created_at: existing.as_ref().map(|a| a.created_at).unwrap_or(now),
            updated_at: now,
        };

        if !cmd.credentials.is_empty() {
            let mut merged = self
                .vault
                .load_credentials(&id)
                .await
                .unwrap_or_default();
            for (k, v) in cmd.credentials {
                if !v.trim().is_empty() {
                    merged.fields.insert(k, v.trim().to_string());
                }
            }
            self.vault
                .save_credentials(&id, &ProviderCredentials { fields: merged.fields })
                .await?;
        }

        self.repository.save_account(account.clone()).await?;

        let envelope = DomainEventEnvelope::wrap(DomainEvent::ProviderConfigured {
            account: account.clone(),
        });
        self.repository.append_event(envelope.clone()).await?;
        self.projector.apply_event(&envelope).await;
        self.publisher.publish(envelope).await;

        Ok(account)
    }
}
