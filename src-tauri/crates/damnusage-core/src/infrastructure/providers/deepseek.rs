use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;
use uuid::Uuid;

pub struct DeepSeekAdapter {
    client: Client,
}

impl DeepSeekAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for DeepSeekAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::DeepSeek
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let api_key = credentials
            .get("api_key")
            .ok_or_else(|| DomainError::MissingCredential("api_key".to_string()))?;

        let base = account
            .endpoint_override
            .as_deref()
            .unwrap_or("https://api.deepseek.com");
        let url = format!("{}/user/balance", base.trim_end_matches('/'));

        let resp = self
            .client
            .get(&url)
            .bearer_auth(api_key)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "deepseek".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "deepseek".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let json_val: Value =
            resp.json()
                .await
                .map_err(|e| DomainError::ProviderParseFailed {
                    provider: "deepseek".to_string(),
                    message: e.to_string(),
                })?;

        let mut total_usd_balance = 0.0_f64;
        let mut windows = Vec::new();

        if let Some(infos) = json_val.get("balance_infos").and_then(|v| v.as_array()) {
            for info in infos {
                let currency = info
                    .get("currency")
                    .and_then(|v| v.as_str())
                    .unwrap_or("USD");
                let total = info
                    .get("total_balance")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let granted = info
                    .get("granted_balance")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);
                let topped = info
                    .get("topped_up_balance")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0);

                let usd_equiv = if currency == "CNY" {
                    total / 7.24
                } else {
                    total
                };
                total_usd_balance += usd_equiv;

                windows.push(QuotaWindow {
                    label: format!("Balance ({currency}) · Top-up {topped:.2} + Grant {granted:.2}"),
                    used: 0.0,
                    limit: total.max(0.01),
                    unit: currency.to_string(),
                    resets_at: None,
                });
            }
        }

        let budget = account.monthly_budget_usd.unwrap_or(30.0);
        let spend_est = (budget - total_usd_balance).max(0.0);
        let utilization_pct = if budget > 0.0 {
            ((spend_est / budget) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::DeepSeek,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: spend_est,
            daily_spend_usd: spend_est / 30.0,
            balance_usd: Some(total_usd_balance),
            monthly_limit_usd: Some(budget),
            utilization_pct,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: 0,
            },
            quota_windows: windows,
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.deepseek.com/user/balance".to_string(),
        })
    }
}
