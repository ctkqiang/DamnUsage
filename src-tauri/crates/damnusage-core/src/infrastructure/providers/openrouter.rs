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

pub struct OpenRouterAdapter {
    client: Client,
}

impl OpenRouterAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for OpenRouterAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenRouter
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
            .unwrap_or("https://openrouter.ai/api/v1")
            .trim_end_matches('/');

        let key_resp = self
            .client
            .get(format!("{base}/auth/key"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "openrouter".to_string(),
                message: e.to_string(),
            })?;

        if !key_resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "openrouter".to_string(),
                message: format!("HTTP {}", key_resp.status()),
            });
        }

        let key_json: Value =
            key_resp
                .json()
                .await
                .map_err(|e| DomainError::ProviderParseFailed {
                    provider: "openrouter".to_string(),
                    message: e.to_string(),
                })?;

        let data = key_json.get("data").unwrap_or(&key_json);
        let usage_total = data.get("usage").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let usage_daily = data
            .get("usage_daily")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let usage_weekly = data
            .get("usage_weekly")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let usage_monthly = data
            .get("usage_monthly")
            .and_then(|v| v.as_f64())
            .unwrap_or(usage_total);
        let limit = data
            .get("limit")
            .and_then(|v| v.as_f64())
            .or(account.monthly_budget_usd);
        let limit_remaining = data.get("limit_remaining").and_then(|v| v.as_f64());

        let mut balance_usd = limit_remaining;
        if let Ok(cred_resp) = self
            .client
            .get(format!("{base}/credits"))
            .bearer_auth(api_key)
            .send()
            .await
        {
            if cred_resp.status().is_success() {
                if let Ok(cred_json) = cred_resp.json::<Value>().await {
                    let cdata = cred_json.get("data").unwrap_or(&cred_json);
                    if let (Some(total_credits), Some(total_usage)) = (
                        cdata.get("total_credits").and_then(|v| v.as_f64()),
                        cdata.get("total_usage").and_then(|v| v.as_f64()),
                    ) {
                        balance_usd = Some((total_credits - total_usage).max(0.0));
                    }
                }
            }
        }

        let eff_limit = limit.unwrap_or(100.0).max(0.01);
        let utilization_pct = ((usage_monthly / eff_limit) * 100.0).clamp(0.0, 100.0);

        let mut windows = vec![QuotaWindow {
            label: "Monthly Usage".to_string(),
            used: usage_monthly,
            limit: eff_limit,
            unit: "USD".to_string(),
            resets_at: None,
        }];
        if usage_weekly > 0.0 {
            windows.push(QuotaWindow {
                label: "Weekly Usage".to_string(),
                used: usage_weekly,
                limit: eff_limit,
                unit: "USD".to_string(),
                resets_at: None,
            });
        }

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::OpenRouter,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: usage_monthly,
            daily_spend_usd: usage_daily,
            balance_usd,
            monthly_limit_usd: limit,
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
            source_note: "openrouter.ai/api/v1/auth/key".to_string(),
        })
    }
}
