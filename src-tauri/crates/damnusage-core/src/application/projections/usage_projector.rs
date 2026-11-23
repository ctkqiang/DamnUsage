use crate::application::queries::get_dashboard::{
    DashboardReadModel, NotchHeadlineReadModel, ProviderSummaryReadModel,
};
use crate::application::queries::get_provider_detail::ProviderDetailReadModel;
use crate::application::queries::list_providers::ProviderCatalogItemView;
use crate::domain::catalog::{catalog_entry_for, full_provider_catalog};
use crate::domain::entities::{ProviderAccount, UsageSnapshot, UserPreferences};
use crate::domain::events::{DomainEvent, DomainEventEnvelope};
use crate::domain::value_objects::{
    AlertSeverity, NotchDisplayState, SparklinePoint, SyncStatus, UsageAlert,
};
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
struct ProjectedState {
    accounts: HashMap<String, ProviderAccount>,
    latest_snapshots: HashMap<String, UsageSnapshot>,
    history: HashMap<String, Vec<SparklinePoint>>,
    alerts: Vec<UsageAlert>,
    preferences: UserPreferences,
    notch_state: NotchDisplayState,
    last_synced_at: Option<String>,
}

impl Default for NotchDisplayState {
    fn default() -> Self {
        Self::Collapsed
    }
}

pub struct UsageProjector {
    state: RwLock<ProjectedState>,
}

impl UsageProjector {
    pub fn new(initial_preferences: UserPreferences) -> Self {
        Self {
            state: RwLock::new(ProjectedState {
                preferences: initial_preferences,
                ..ProjectedState::default()
            }),
        }
    }

    pub async fn hydrate(
        &self,
        accounts: Vec<ProviderAccount>,
        events: Vec<DomainEventEnvelope>,
        preferences: UserPreferences,
    ) {
        let mut guard = self.state.write().await;
        guard.accounts.clear();
        guard.latest_snapshots.clear();
        guard.history.clear();
        guard.alerts.clear();
        guard.preferences = preferences;

        for account in accounts {
            guard.accounts.insert(account.id.clone(), account);
        }

        for envelope in &events {
            Self::reduce(&mut guard, envelope);
        }
    }

    pub async fn apply_event(&self, envelope: &DomainEventEnvelope) {
        let mut guard = self.state.write().await;
        Self::reduce(&mut guard, envelope);
    }

    fn reduce(state: &mut ProjectedState, envelope: &DomainEventEnvelope) {
        match &envelope.payload {
            DomainEvent::ProviderConfigured { account } => {
                state.accounts.insert(account.id.clone(), account.clone());
            }
            DomainEvent::ProviderRemoved { provider_id } => {
                state.accounts.remove(provider_id);
                state.latest_snapshots.remove(provider_id);
                state.history.remove(provider_id);
                state.alerts.retain(|a| &a.provider_id != provider_id);
            }
            DomainEvent::UsageSnapshotRecorded { snapshot } => {
                state.last_synced_at = Some(snapshot.recorded_at.to_rfc3339());
                let series = state
                    .history
                    .entry(snapshot.provider_id.clone())
                    .or_default();
                series.push(SparklinePoint {
                    timestamp: snapshot.recorded_at.to_rfc3339(),
                    spend_usd: snapshot.spend_usd,
                    tokens: snapshot.tokens.total_tokens(),
                });
                if series.len() > 36 {
                    let excess = series.len() - 36;
                    series.drain(0..excess);
                }
                state
                    .latest_snapshots
                    .insert(snapshot.provider_id.clone(), snapshot.clone());
            }
            DomainEvent::BudgetThresholdBreached {
                provider_id,
                provider_name,
                utilization_pct,
                severity,
            } => {
                state.alerts.retain(|a| &a.provider_id != provider_id);
                state.alerts.push(UsageAlert {
                    id: envelope.event_id.clone(),
                    provider_id: provider_id.clone(),
                    provider_name: provider_name.clone(),
                    severity: *severity,
                    message_key: match severity {
                        AlertSeverity::Critical => "alerts.critical_quota".to_string(),
                        AlertSeverity::Warning => "alerts.warning_quota".to_string(),
                        AlertSeverity::Info => "alerts.info_quota".to_string(),
                    },
                    utilization_pct: *utilization_pct,
                    triggered_at: envelope.occurred_at.to_rfc3339(),
                });
                if state.alerts.len() > 12 {
                    state.alerts.drain(0..state.alerts.len() - 12);
                }
            }
            DomainEvent::PreferencesUpdated { preferences } => {
                state.preferences = preferences.clone();
            }
            DomainEvent::NotchStateChanged { state: next } => {
                state.notch_state = *next;
            }
        }
    }

    pub async fn read_dashboard(&self) -> DashboardReadModel {
        let guard = self.state.read().await;
        let prefs = &guard.preferences;

        let mut provider_cards: Vec<ProviderSummaryReadModel> = Vec::new();
        let mut total_spend_usd = 0.0_f64;
        let mut total_daily_spend_usd = 0.0_f64;
        let mut total_tokens = 0_u64;
        let mut total_requests = 0_u64;

        let mut account_list: Vec<&ProviderAccount> = guard.accounts.values().collect();
        account_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        for account in account_list {
            if !account.enabled {
                continue;
            }
            let spec = catalog_entry_for(account.kind);
            let snap = guard.latest_snapshots.get(&account.id);
            let sparkline = guard
                .history
                .get(&account.id)
                .cloned()
                .unwrap_or_default();

            let (spend, daily, balance, limit, util, tokens, reqs, status, updated, source_note) =
                match snap {
                    Some(s) => (
                        s.spend_usd,
                        s.daily_spend_usd,
                        s.balance_usd,
                        s.monthly_limit_usd.or(account.monthly_budget_usd),
                        s.utilization_pct,
                        s.tokens.total_tokens(),
                        s.tokens.total_requests,
                        s.status,
                        Some(s.recorded_at.to_rfc3339()),
                        s.source_note.clone(),
                    ),
                    None => (
                        0.0,
                        0.0,
                        None,
                        account.monthly_budget_usd,
                        0.0,
                        0,
                        0,
                        SyncStatus::Syncing,
                        None,
                        String::new(),
                    ),
                };

            total_spend_usd += spend;
            total_daily_spend_usd += daily;
            total_tokens = total_tokens.saturating_add(tokens);
            total_requests = total_requests.saturating_add(reqs);

            provider_cards.push(ProviderSummaryReadModel {
                provider_id: account.id.clone(),
                kind: account.kind,
                label: account.label.clone(),
                short_name: spec.short_name,
                category: spec.category,
                accent_hex: spec.accent_hex,
                secondary_hex: spec.secondary_hex,
                status,
                spend_usd: spend,
                daily_spend_usd: daily,
                balance_usd: balance,
                monthly_limit_usd: limit,
                utilization_pct: util,
                total_tokens: tokens,
                total_requests: reqs,
                last_updated: updated,
                source_note,
                sparkline,
            });
        }

        provider_cards.sort_by(|a, b| {
            b.spend_usd
                .partial_cmp(&a.spend_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let global_util = if prefs.monthly_global_budget_usd > 0.0 {
            ((total_spend_usd / prefs.monthly_global_budget_usd) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let lead = provider_cards.first();
        let headline = NotchHeadlineReadModel {
            primary_provider_name: lead
                .map(|p| p.short_name.clone())
                .unwrap_or_else(|| "DamnUsage".to_string()),
            primary_accent_hex: lead
                .map(|p| p.accent_hex.clone())
                .unwrap_or_else(|| "#7C5CFF".to_string()),
            total_spend_usd,
            total_spend_cny: total_spend_usd * prefs.usd_to_cny_rate,
            global_utilization_pct: global_util,
            active_provider_count: provider_cards.len(),
            alert_count: guard.alerts.len(),
            status: if guard
                .alerts
                .iter()
                .any(|a| a.severity == AlertSeverity::Critical)
            {
                SyncStatus::Critical
            } else if guard
                .alerts
                .iter()
                .any(|a| a.severity == AlertSeverity::Warning)
            {
                SyncStatus::Warning
            } else if prefs.demo_mode && provider_cards.iter().any(|p| p.status == SyncStatus::Simulated) {
                SyncStatus::Simulated
            } else {
                SyncStatus::Healthy
            },
        };

        let aggregate_sparkline = Self::build_aggregate_sparkline(&provider_cards);

        DashboardReadModel {
            generated_at: Utc::now().to_rfc3339(),
            last_synced_at: guard.last_synced_at.clone(),
            notch_state: guard.notch_state,
            demo_mode: prefs.demo_mode,
            currency: prefs.currency,
            usd_to_cny_rate: prefs.usd_to_cny_rate,
            monthly_global_budget_usd: prefs.monthly_global_budget_usd,
            total_spend_usd,
            total_spend_cny: total_spend_usd * prefs.usd_to_cny_rate,
            daily_burn_rate_usd: total_daily_spend_usd,
            total_tokens,
            total_requests,
            global_utilization_pct: global_util,
            headline,
            providers: provider_cards,
            alerts: guard.alerts.clone(),
            aggregate_sparkline,
        }
    }

    fn build_aggregate_sparkline(cards: &[ProviderSummaryReadModel]) -> Vec<SparklinePoint> {
        let max_len = cards.iter().map(|c| c.sparkline.len()).max().unwrap_or(0);
        if max_len == 0 {
            return Vec::new();
        }
        (0..max_len)
            .map(|idx| {
                let mut spend = 0.0;
                let mut tokens = 0_u64;
                let mut ts = String::new();
                for card in cards {
                    if let Some(pt) = card.sparkline.get(idx) {
                        spend += pt.spend_usd;
                        tokens = tokens.saturating_add(pt.tokens);
                        if ts.is_empty() {
                            ts = pt.timestamp.clone();
                        }
                    }
                }
                SparklinePoint {
                    timestamp: ts,
                    spend_usd: spend,
                    tokens,
                }
            })
            .collect()
    }

    pub async fn read_provider_detail(&self, provider_id: &str) -> Option<ProviderDetailReadModel> {
        let guard = self.state.read().await;
        let account = guard.accounts.get(provider_id)?;
        let spec = catalog_entry_for(account.kind);
        let snapshot = guard.latest_snapshots.get(provider_id).cloned();
        let history = guard
            .history
            .get(provider_id)
            .cloned()
            .unwrap_or_default();

        Some(ProviderDetailReadModel {
            account: account.clone(),
            catalog: spec,
            snapshot,
            history,
        })
    }

    pub async fn read_catalog(&self) -> Vec<ProviderCatalogItemView> {
        let guard = self.state.read().await;
        full_provider_catalog()
            .into_iter()
            .map(|entry| {
                let configured_ids: Vec<String> = guard
                    .accounts
                    .values()
                    .filter(|acc| acc.kind == entry.kind)
                    .map(|acc| acc.id.clone())
                    .collect();
                ProviderCatalogItemView {
                    configured_count: configured_ids.len(),
                    configured_ids,
                    entry,
                }
            })
            .collect()
    }

    pub async fn read_preferences(&self) -> UserPreferences {
        self.state.read().await.preferences.clone()
    }

    pub async fn has_accounts(&self) -> bool {
        !self.state.read().await.accounts.is_empty()
    }
}
