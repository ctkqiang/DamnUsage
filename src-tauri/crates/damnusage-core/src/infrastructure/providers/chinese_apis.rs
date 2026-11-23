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

pub struct SiliconFlowAdapter {
    client: Client,
}

impl SiliconFlowAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for SiliconFlowAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::SiliconFlow
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
            .unwrap_or("https://api.siliconflow.cn/v1")
            .trim_end_matches('/');

        let resp = self
            .client
            .get(format!("{base}/user/info"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "siliconflow".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "siliconflow".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "siliconflow".to_string(),
                message: e.to_string(),
            })?;

        let data = val.get("data").unwrap_or(&val);
        let total_cny = parse_num(data.get("totalBalance"));
        let charge_cny = parse_num(data.get("chargeBalance"));
        let gift_cny = parse_num(data.get("balance"));
        let balance_usd = total_cny / 7.24;
        let budget = account.monthly_budget_usd.unwrap_or(20.0);
        let spend = (budget - balance_usd).max(0.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::SiliconFlow,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: spend,
            daily_spend_usd: spend / 30.0,
            balance_usd: Some(balance_usd),
            monthly_limit_usd: Some(budget),
            utilization_pct: ((spend / budget.max(0.01)) * 100.0).clamp(0.0, 100.0),
            tokens: empty_tokens(),
            quota_windows: vec![QuotaWindow {
                label: format!("CNY Balance · Recharge {charge_cny:.2} + Gift {gift_cny:.2}"),
                used: 0.0,
                limit: total_cny.max(0.01),
                unit: "CNY".to_string(),
                resets_at: None,
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.siliconflow.cn/v1/user/info".to_string(),
        })
    }
}

pub struct MoonshotKimiAdapter {
    client: Client,
}

impl MoonshotKimiAdapter {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl UsageProviderPort for MoonshotKimiAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::MoonshotKimi
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
            .unwrap_or("https://api.moonshot.cn/v1")
            .trim_end_matches('/');

        let resp = self
            .client
            .get(format!("{base}/users/me/balance"))
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: "moonshot_kimi".to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: "moonshot_kimi".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let val: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::ProviderParseFailed {
                provider: "moonshot_kimi".to_string(),
                message: e.to_string(),
            })?;

        let data = val.get("data").unwrap_or(&val);
        let available_cny = parse_num(data.get("available_balance"));
        let voucher_cny = parse_num(data.get("voucher_balance"));
        let cash_cny = parse_num(data.get("cash_balance"));
        let balance_usd = available_cny / 7.24;
        let budget = account.monthly_budget_usd.unwrap_or(25.0);
        let spend = (budget - balance_usd).max(0.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::MoonshotKimi,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: spend,
            daily_spend_usd: spend / 30.0,
            balance_usd: Some(balance_usd),
            monthly_limit_usd: Some(budget),
            utilization_pct: ((spend / budget.max(0.01)) * 100.0).clamp(0.0, 100.0),
            tokens: empty_tokens(),
            quota_windows: vec![QuotaWindow {
                label: format!("Kimi Balance · Cash {cash_cny:.2} + Voucher {voucher_cny:.2}"),
                used: 0.0,
                limit: available_cny.max(0.01),
                unit: "CNY".to_string(),
                resets_at: None,
            }],
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: "api.moonshot.cn/v1/users/me/balance".to_string(),
        })
    }
}

pub struct GenericBearerProbeAdapter {
    client: Client,
    kind: ProviderKind,
    default_url: &'static str,
    source_label: &'static str,
}

impl GenericBearerProbeAdapter {
    pub fn new(
        client: Client,
        kind: ProviderKind,
        default_url: &'static str,
        source_label: &'static str,
    ) -> Self {
        Self {
            client,
            kind,
            default_url,
            source_label,
        }
    }
}

#[async_trait]
impl UsageProviderPort for GenericBearerProbeAdapter {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let api_key = credentials
            .get("api_key")
            .or_else(|| credentials.get("oauth_token"))
            .ok_or_else(|| DomainError::MissingCredential("api_key".to_string()))?;

        let url = account
            .endpoint_override
            .as_deref()
            .unwrap_or(self.default_url);

        let resp = self
            .client
            .get(url)
            .bearer_auth(api_key)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| DomainError::ProviderRequestFailed {
                provider: self.kind.to_string(),
                message: e.to_string(),
            })?;

        if !resp.status().is_success() {
            return Err(DomainError::ProviderRequestFailed {
                provider: self.kind.to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        let remaining_req = resp
            .headers()
            .get("x-ratelimit-remaining-requests")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());
        let limit_req = resp
            .headers()
            .get("x-ratelimit-limit-requests")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<f64>().ok());

        let body_val = resp.json::<Value>().await.unwrap_or(Value::Null);
        let balance = extract_first_number(
            &body_val,
            &[
                "/balance",
                "/data/balance",
                "/credits",
                "/remaining_credits",
                "/total_available",
            ],
        );
        let spend = extract_first_number(
            &body_val,
            &[
                "/total_usage",
                "/usage",
                "/data/usage",
                "/spend",
                "/total_cost",
            ],
        )
        .unwrap_or(0.0);

        let budget = account.monthly_budget_usd.unwrap_or(50.0);
        let util = if let (Some(rem), Some(lim)) = (remaining_req, limit_req) {
            if lim > 0.0 {
                (((lim - rem) / lim) * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            }
        } else {
            ((spend / budget.max(0.01)) * 100.0).clamp(0.0, 100.0)
        };

        let mut windows = Vec::new();
        if let (Some(rem), Some(lim)) = (remaining_req, limit_req) {
            windows.push(QuotaWindow {
                label: "Rate Limit Window".to_string(),
                used: (lim - rem).max(0.0),
                limit: lim,
                unit: "RPM".to_string(),
                resets_at: None,
            });
        }

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: self.kind,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd: spend,
            daily_spend_usd: spend / 30.0,
            balance_usd: balance,
            monthly_limit_usd: Some(budget),
            utilization_pct: util,
            tokens: empty_tokens(),
            quota_windows: windows,
            model_breakdown: Vec::new(),
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: self.source_label.to_string(),
        })
    }
}

fn parse_num(val: Option<&Value>) -> f64 {
    match val {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    }
}

fn extract_first_number(val: &Value, pointers: &[&str]) -> Option<f64> {
    for ptr in pointers {
        if let Some(found) = val.pointer(ptr) {
            let n = parse_num(Some(found));
            if n > 0.0 {
                return Some(n);
            }
        }
    }
    None
}

fn empty_tokens() -> TokenMetrics {
    TokenMetrics {
        input_tokens: 0,
        output_tokens: 0,
        cached_tokens: 0,
        total_requests: 0,
    }
}
