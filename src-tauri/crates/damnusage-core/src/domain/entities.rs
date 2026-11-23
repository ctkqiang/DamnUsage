use crate::domain::value_objects::{
    CurrencyCode, LocaleCode, ModelUsageBreakdown, ProviderKind, QuotaWindow, SyncStatus,
    TokenMetrics,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAccount {
    pub id: String,
    pub kind: ProviderKind,
    pub label: String,
    pub enabled: bool,
    pub monthly_budget_usd: Option<f64>,
    pub endpoint_override: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub snapshot_id: String,
    pub provider_id: String,
    pub kind: ProviderKind,
    pub recorded_at: DateTime<Utc>,
    pub status: SyncStatus,
    pub spend_usd: f64,
    pub daily_spend_usd: f64,
    pub balance_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
    pub utilization_pct: f64,
    pub tokens: TokenMetrics,
    pub quota_windows: Vec<QuotaWindow>,
    pub model_breakdown: Vec<ModelUsageBreakdown>,
    pub latency_ms: u64,
    pub error_message: Option<String>,
    pub source_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub locale: LocaleCode,
    pub currency: CurrencyCode,
    pub usd_to_cny_rate: f64,
    pub refresh_interval_secs: u64,
    pub monthly_global_budget_usd: f64,
    pub warning_threshold_pct: f64,
    pub critical_threshold_pct: f64,
    pub demo_mode: bool,
    pub hover_to_peek: bool,
    pub show_cost_in_notch: bool,
    pub notch_width_offset_px: i32,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            locale: LocaleCode::En,
            currency: CurrencyCode::Usd,
            usd_to_cny_rate: 7.24,
            refresh_interval_secs: 60,
            monthly_global_budget_usd: 250.0,
            warning_threshold_pct: 75.0,
            critical_threshold_pct: 90.0,
            demo_mode: true,
            hover_to_peek: true,
            show_cost_in_notch: true,
            notch_width_offset_px: 0,
        }
    }
}
