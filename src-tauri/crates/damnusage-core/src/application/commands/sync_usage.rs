use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::ports::{
    EventPublisherPort, SecretVaultPort, UsageProviderPort, WriteRepositoryPort,
};
use crate::domain::value_objects::{AlertSeverity, ProviderKind, SyncStatus, TokenMetrics};
use chrono::Utc;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProviderUsageCommand {
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncAllProvidersCommand {
    pub include_demo_if_empty: bool,
}

pub struct SyncUsageHandler {
    repository: Arc<dyn WriteRepositoryPort>,
    vault: Arc<dyn SecretVaultPort>,
    providers: HashMap<ProviderKind, Arc<dyn UsageProviderPort>>,
    demo_provider: Arc<dyn UsageProviderPort>,
    projector: Arc<UsageProjector>,
    publisher: Arc<dyn EventPublisherPort>,
}

impl SyncUsageHandler {
    pub fn new(
        repository: Arc<dyn WriteRepositoryPort>,
        vault: Arc<dyn SecretVaultPort>,
        providers: HashMap<ProviderKind, Arc<dyn UsageProviderPort>>,
        demo_provider: Arc<dyn UsageProviderPort>,
        projector: Arc<UsageProjector>,
        publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            repository,
            vault,
            providers,
            demo_provider,
            projector,
            publisher,
        }
    }

    pub async fn handle_one(&self, cmd: SyncProviderUsageCommand) -> DomainResult<UsageSnapshot> {
        let account = self
            .repository
            .get_account(&cmd.provider_id)
            .await?
            .ok_or_else(|| DomainError::ProviderNotFound(cmd.provider_id.clone()))?;

        self.sync_account(&account).await
    }

    pub async fn handle_all(&self, _cmd: SyncAllProvidersCommand) -> DomainResult<Vec<UsageSnapshot>> {
        let accounts: Vec<ProviderAccount> = self
            .repository
            .list_accounts()
            .await?
            .into_iter()
            .filter(|a| a.enabled)
            .collect();

        let futures = accounts.iter().map(|acc| self.sync_account(acc));
        let results = join_all(futures).await;

        let mut snapshots = Vec::new();
        for res in results {
            if let Ok(snap) = res {
                snapshots.push(snap);
            }
        }
        Ok(snapshots)
    }

    async fn sync_account(&self, account: &ProviderAccount) -> DomainResult<UsageSnapshot> {
        let prefs = self.projector.read_preferences().await;
        let is_demo_account = account
            .metadata
            .get("demo")
            .map(|v| v == "true")
            .unwrap_or(false);

        let snapshot = if is_demo_account || prefs.demo_mode && self.vault.load_credentials(&account.id).await.map(|c| c.fields.is_empty()).unwrap_or(true) {
            let creds = self.vault.load_credentials(&account.id).await.unwrap_or_default();
            if is_demo_account {
                self.demo_provider.fetch_usage(account, &creds).await?
            } else if let Some(adapter) = self.providers.get(&account.kind) {
                match adapter.fetch_usage(account, &creds).await {
                    Ok(snap) => snap,
                    Err(_) if prefs.demo_mode => self.demo_provider.fetch_usage(account, &creds).await?,
                    Err(err) => self.error_snapshot(account, err.to_string()),
                }
            } else {
                self.demo_provider.fetch_usage(account, &creds).await?
            }
        } else {
            let creds = self
                .vault
                .load_credentials(&account.id)
                .await
                .unwrap_or_default();
            let adapter = self
                .providers
                .get(&account.kind)
                .ok_or_else(|| DomainError::UnsupportedProvider(account.kind.to_string()))?;

            match adapter.fetch_usage(account, &creds).await {
                Ok(mut snap) => {
                    if let Some(budget) = account.monthly_budget_usd {
                        if budget > 0.0 && snap.monthly_limit_usd.is_none() {
                            snap.monthly_limit_usd = Some(budget);
                            snap.utilization_pct = ((snap.spend_usd / budget) * 100.0).clamp(0.0, 100.0);
                        }
                    }
                    snap
                }
                Err(err) => self.error_snapshot(account, err.to_string()),
            }
        };

        let envelope = DomainEventEnvelope::wrap(DomainEvent::UsageSnapshotRecorded {
            snapshot: snapshot.clone(),
        });
        self.repository.append_event(envelope.clone()).await?;
        self.projector.apply_event(&envelope).await;
        self.publisher.publish(envelope).await;

        if snapshot.utilization_pct >= prefs.critical_threshold_pct {
            let alert = DomainEventEnvelope::wrap(DomainEvent::BudgetThresholdBreached {
                provider_id: account.id.clone(),
                provider_name: account.label.clone(),
                utilization_pct: snapshot.utilization_pct,
                severity: AlertSeverity::Critical,
            });
            self.projector.apply_event(&alert).await;
            self.publisher.publish(alert).await;
        } else if snapshot.utilization_pct >= prefs.warning_threshold_pct {
            let alert = DomainEventEnvelope::wrap(DomainEvent::BudgetThresholdBreached {
                provider_id: account.id.clone(),
                provider_name: account.label.clone(),
                utilization_pct: snapshot.utilization_pct,
                severity: AlertSeverity::Warning,
            });
            self.projector.apply_event(&alert).await;
            self.publisher.publish(alert).await;
        }

        Ok(snapshot)
    }

    fn error_snapshot(&self, account: &ProviderAccount, message: String) -> UsageSnapshot {
        UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: account.kind,
            recorded_at: Utc::now(),
            status: SyncStatus::Error,
            spend_usd: 0.0,
            daily_spend_usd: 0.0,
            balance_usd: None,
            monthly_limit_usd: account.monthly_budget_usd,
            utilization_pct: 0.0,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: 0,
            },
            quota_windows: Vec::new(),
            model_breakdown: Vec::new(),
            latency_ms: 0,
            error_message: Some(message),
            source_note: "sync_error".to_string(),
        }
    }
}
