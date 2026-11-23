use crate::domain::entities::{ProviderAccount, UsageSnapshot, UserPreferences};
use crate::domain::value_objects::{AlertSeverity, NotchDisplayState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEventEnvelope {
    pub event_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: DomainEvent,
}

impl DomainEventEnvelope {
    pub fn wrap(payload: DomainEvent) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            occurred_at: Utc::now(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    ProviderConfigured {
        account: ProviderAccount,
    },
    ProviderRemoved {
        provider_id: String,
    },
    UsageSnapshotRecorded {
        snapshot: UsageSnapshot,
    },
    BudgetThresholdBreached {
        provider_id: String,
        provider_name: String,
        utilization_pct: f64,
        severity: AlertSeverity,
    },
    PreferencesUpdated {
        preferences: UserPreferences,
    },
    NotchStateChanged {
        state: NotchDisplayState,
    },
}
