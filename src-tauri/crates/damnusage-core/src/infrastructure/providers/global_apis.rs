use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use async_trait::async_trait;
use chrono::{Datelike, TimeZone, Utc};
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;
use uuid::Uuid;

pub struct OpenAiAdapter {
    client: Client,
}

impl OpenAiAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for OpenAiAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAi
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
            .unwrap_or("https://api.openai.com/v1")
            .trim_end_matches('/');

        let now = Utc::now();
        let month_start = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .single()
            .unwrap_or(now)
            .timestamp();

        let mut req = self
            .client
            .get(format!("{base}/organization/costs?start_time={month_start}&limit=31"))
            .bearer_auth(api_key);
        if let Some(org) = credentials.get("org_id") {
            req = req.header("OpenAI-Organization", org);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "openai".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "openai".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "openai".to_string(),
                message: e.to_string(),
            })?;

        let mut total_spend = 0.0_f64;
        let mut latest_daily = 0.0_f64;
        if let Some(buckets) = val.get("data").and_then(|v| v.as_array()) {
            for bucket in buckets {
                let mut bucket_sum = 0.0;
                if let Some(results) = bucket.get("results").and_then(|v| v.as_array()) {
                    for item in results {
                        let amount = item
                            .pointer("/amount/value")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        bucket_sum += amount;
                    }
                }
                total_spend += bucket_sum;
                latest_daily = bucket_sum;
            }
        }

        let budget = account.monthly_budget_usd.unwrap_or(75.0);
        let util = ((total_spend / budget.max(0.01)) * 100.0).clamp(0.0, 100.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::OpenAi,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: total_spend,
            daily_spend_usd: latest_daily,
            balance_usd: Some((budget - total_spend).max(0.0)),
            monthly_limit_usd: Some(budget),
            utilization_pct: util,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: 0,
            },
            quota_windows: vec![QuotaWindow {
                label: "Monthly Organization Costs".to_string(),
                used: total_spend,
                limit: budget,
                unit: "USD".to_string(),
                resets_at: None,
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.openai.com/v1/organization/costs".to_string(),
        })
    }
}

pub struct AnthropicApiAdapter {
    client: Client,
}

impl AnthropicApiAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for AnthropicApiAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::AnthropicApi
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
            .unwrap_or("https://api.anthropic.com/v1")
            .trim_end_matches('/');

        let now = Utc::now();
        let start_iso = format!("{:04}-{:02}-01T00:00:00Z", now.year(), now.month());
        let end_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let resp = self
            .client
            .get(format!(
                "{base}/organizations/cost_report?starting_at={start_iso}&ending_at={end_iso}&bucket_width=1d"
            ))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "anthropic_api".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "anthropic_api".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "anthropic_api".to_string(),
                message: e.to_string(),
            })?;

        let mut total_cents = 0.0_f64;
        let mut latest_day_cents = 0.0_f64;
        if let Some(buckets) = val.get("data").and_then(|v| v.as_array()) {
            for bucket in buckets {
                let mut day_c = 0.0;
                if let Some(results) = bucket.get("results").and_then(|v| v.as_array()) {
                    for row in results {
                        let amount = row
                            .get("amount")
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        day_c += amount;
                    }
                }
                total_cents += day_c;
                latest_day_cents = day_c;
            }
        }

        let spend_usd = total_cents / 100.0;
        let daily_usd = latest_day_cents / 100.0;
        let budget = account.monthly_budget_usd.unwrap_or(100.0);
        let util = ((spend_usd / budget.max(0.01)) * 100.0).clamp(0.0, 100.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::AnthropicApi,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd,
            daily_spend_usd: daily_usd,
            balance_usd: Some((budget - spend_usd).max(0.0)),
            monthly_limit_usd: Some(budget),
            utilization_pct: util,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: 0,
            },
            quota_windows: vec![QuotaWindow {
                label: "Anthropic Org Cost Report".to_string(),
                used: spend_usd,
                limit: budget,
                unit: "USD".to_string(),
                resets_at: None,
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.anthropic.com/v1/organizations/cost_report".to_string(),
        })
    }
}

pub struct CursorIdeAdapter {
    client: Client,
}

impl CursorIdeAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for CursorIdeAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::CursorIde
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let token = credentials
            .get("session_token")
            .ok_or_else(|| DomainError::MissingCredential("session_token".to_string()))?;
        let user_id = credentials.get("user_id").unwrap_or("");
        let url = if user_id.is_empty() {
            "https://www.cursor.com/api/usage".to_string()
        } else {
            format!("https://www.cursor.com/api/usage?user={user_id}")
        };

        let resp = self
            .client
            .get(&url)
            .header("Cookie", format!("WorkosCursorSessionToken={token}"))
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "cursor_ide".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "cursor_ide".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "cursor_ide".to_string(),
                message: e.to_string(),
            })?;

        let fast_used = val
            .pointer("/gpt-4/numRequests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let fast_limit = val
            .pointer("/gpt-4/maxRequestUsage")
            .and_then(|v| v.as_u64())
            .unwrap_or(500);

        let util = if fast_limit > 0 {
            ((fast_used as f64 / fast_limit as f64) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let plan_cost = account.monthly_budget_usd.unwrap_or(20.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::CursorIde,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: plan_cost * (util / 100.0),
            daily_spend_usd: (plan_cost / 30.0),
            balance_usd: None,
            monthly_limit_usd: Some(plan_cost),
            utilization_pct: util,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: fast_used,
            },
            quota_windows: vec![QuotaWindow {
                label: "Fast Premium Requests".to_string(),
                used: fast_used as f64,
                limit: fast_limit as f64,
                unit: "req".to_string(),
                resets_at: None,
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "cursor.com/api/usage".to_string(),
        })
    }
}

pub struct GithubCopilotAdapter {
    client: Client,
}

impl GithubCopilotAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for GithubCopilotAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::GithubCopilot
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let token = credentials
            .get("oauth_token")
            .ok_or_else(|| DomainError::MissingCredential("oauth_token".to_string()))?;

        let resp = self
            .client
            .get("https://api.github.com/copilot_internal/user")
            .bearer_auth(token)
            .header("User-Agent", "DamnUsage/0.1")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "github_copilot".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "github_copilot".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "github_copilot".to_string(),
                message: e.to_string(),
            })?;

        let remaining_pct = val
            .pointer("/quota_snapshots/premium_interactions/percent_remaining")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);
        let used_pct = (100.0 - remaining_pct).clamp(0.0, 100.0);
        let plan_budget = account.monthly_budget_usd.unwrap_or(19.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::GithubCopilot,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: plan_budget * (used_pct / 100.0),
            daily_spend_usd: plan_budget / 30.0,
            balance_usd: None,
            monthly_limit_usd: Some(plan_budget),
            utilization_pct: used_pct,
            tokens: TokenMetrics {
                input_tokens: 0,
                output_tokens: 0,
                cached_tokens: 0,
                total_requests: 0,
            },
            quota_windows: vec![QuotaWindow {
                label: "Premium Interactions".to_string(),
                used: used_pct,
                limit: 100.0,
                unit: "%".to_string(),
                resets_at: val
                    .get("quota_reset_date")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.github.com/copilot_internal/user".to_string(),
        })
    }
}
