use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::DomainResult;
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ModelUsageBreakdown, ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct AntigravityAdapter;

impl AntigravityAdapter {
    pub fn new() -> Self {
        Self
    }

    fn resolve_data_dir(&self, account: &ProviderAccount, credentials: &ProviderCredentials) -> PathBuf {
        if let Some(custom) = credentials
            .get("data_dir")
            .or(account.endpoint_override.as_deref())
        {
            if let Some(stripped) = custom.strip_prefix("~/") {
                if let Some(home) = dirs::home_dir() {
                    return home.join(stripped);
                }
            }
            return PathBuf::from(custom);
        }
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".gemini")
            .join("antigravity-cli")
    }

    fn estimate_cost(model: &str, input: u64, output: u64, cached: u64) -> f64 {
        let lower = model.to_lowercase();
        let (in_rate, out_rate, cache_rate) = if lower.contains("pro") || lower.contains("ultra") || lower.contains("neon") {
            (2.50, 15.00, 0.25)
        } else if lower.contains("flash-lite") {
            (0.10, 0.40, 0.025)
        } else {
            (0.50, 3.00, 0.05)
        };
        (input as f64 / 1_000_000.0) * in_rate
            + (output as f64 / 1_000_000.0) * out_rate
            + (cached as f64 / 1_000_000.0) * cache_rate
    }
}

impl Default for AntigravityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UsageProviderPort for AntigravityAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Antigravity
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let root = self.resolve_data_dir(account, credentials);

        let mut total_input = 0_u64;
        let mut total_output = 0_u64;
        let mut total_cached = 0_u64;
        let mut total_requests = 0_u64;
        let mut by_model: HashMap<String, (u64, u64, u64, f64)> = HashMap::new();

        if root.exists() {
            for entry in WalkDir::new(&root)
                .max_depth(6)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext != "jsonl" && ext != "json" {
                    continue;
                }
                let content = match std::fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                for line in content.lines() {
                    if !line.contains("token") && !line.contains("usage") {
                        continue;
                    }
                    let Ok(val) = serde_json::from_str::<Value>(line) else {
                        continue;
                    };
                    let usage = val
                        .get("usage")
                        .or_else(|| val.pointer("/metadata/usage"))
                        .or_else(|| val.pointer("/usageMetadata"));
                    let Some(u) = usage else {
                        continue;
                    };
                    let input = u
                        .get("input_tokens")
                        .or_else(|| u.get("promptTokenCount"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = u
                        .get("output_tokens")
                        .or_else(|| u.get("candidatesTokenCount"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let cached = u
                        .get("cached_tokens")
                        .or_else(|| u.get("cachedContentTokenCount"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if input == 0 && output == 0 {
                        continue;
                    }
                    let model = val
                        .get("model")
                        .or_else(|| val.pointer("/metadata/model"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("antigravity-default")
                        .to_string();
                    let cost = Self::estimate_cost(&model, input, output, cached);
                    total_input = total_input.saturating_add(input);
                    total_output = total_output.saturating_add(output);
                    total_cached = total_cached.saturating_add(cached);
                    total_requests = total_requests.saturating_add(1);
                    let slot = by_model.entry(model).or_insert((0, 0, 0, 0.0));
                    slot.0 = slot.0.saturating_add(input);
                    slot.1 = slot.1.saturating_add(output);
                    slot.2 = slot.2.saturating_add(1);
                    slot.3 += cost;
                }
            }
        }

        let mut breakdown: Vec<ModelUsageBreakdown> = by_model
            .into_iter()
            .map(|(model_id, (i, o, reqs, cost))| ModelUsageBreakdown {
                model_id,
                spend_usd: cost,
                input_tokens: i,
                output_tokens: o,
                request_count: reqs,
            })
            .collect();
        breakdown.sort_by(|a, b| {
            b.spend_usd
                .partial_cmp(&a.spend_usd)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let spend_usd: f64 = breakdown.iter().map(|b| b.spend_usd).sum();
        let limit = account.monthly_budget_usd.unwrap_or(60.0);
        let util = ((spend_usd / limit) * 100.0).clamp(0.0, 100.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::Antigravity,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd,
            daily_spend_usd: spend_usd / 14.0,
            balance_usd: Some((limit - spend_usd).max(0.0)),
            monthly_limit_usd: Some(limit),
            utilization_pct: util,
            tokens: TokenMetrics {
                input_tokens: total_input,
                output_tokens: total_output,
                cached_tokens: total_cached,
                total_requests,
            },
            quota_windows: vec![QuotaWindow {
                label: "Local Session Telemetry".to_string(),
                used: spend_usd,
                limit,
                unit: "USD".to_string(),
                resets_at: None,
            }],
            model_breakdown: breakdown,
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: format!("local · {}", root.display()),
        })
    }
}
