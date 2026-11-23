use crate::application::projections::usage_projector::UsageProjector;
use crate::domain::value_objects::{
    CurrencyCode, NotchDisplayState, ProviderCategory, ProviderKind, SparklinePoint, SyncStatus,
    UsageAlert,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetDashboardQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotchHeadlineReadModel {
    pub primary_provider_name: String,
    pub primary_accent_hex: String,
    pub total_spend_usd: f64,
    pub total_spend_cny: f64,
    pub global_utilization_pct: f64,
    pub active_provider_count: usize,
    pub alert_count: usize,
    pub status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummaryReadModel {
    pub provider_id: String,
    pub kind: ProviderKind,
    pub label: String,
    pub short_name: String,
    pub category: ProviderCategory,
    pub accent_hex: String,
    pub secondary_hex: String,
    pub status: SyncStatus,
    pub spend_usd: f64,
    pub daily_spend_usd: f64,
    pub balance_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
    pub utilization_pct: f64,
    pub total_tokens: u64,
    pub total_requests: u64,
    pub last_updated: Option<String>,
    pub source_note: String,
    pub sparkline: Vec<SparklinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardReadModel {
    pub generated_at: String,
    pub last_synced_at: Option<String>,
    pub notch_state: NotchDisplayState,
    pub demo_mode: bool,
    pub currency: CurrencyCode,
    pub usd_to_cny_rate: f64,
    pub monthly_global_budget_usd: f64,
    pub total_spend_usd: f64,
    pub total_spend_cny: f64,
    pub daily_burn_rate_usd: f64,
    pub total_tokens: u64,
    pub total_requests: u64,
    pub global_utilization_pct: f64,
    pub headline: NotchHeadlineReadModel,
    pub providers: Vec<ProviderSummaryReadModel>,
    pub alerts: Vec<UsageAlert>,
    pub aggregate_sparkline: Vec<SparklinePoint>,
}

pub struct GetDashboardHandler {
    projector: Arc<UsageProjector>,
}

impl GetDashboardHandler {
    pub fn new(projector: Arc<UsageProjector>) -> Self {
        Self { projector }
    }

    pub async fn handle(&self, _query: GetDashboardQuery) -> DashboardReadModel {
        self.projector.read_dashboard().await
    }
}
