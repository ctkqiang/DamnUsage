use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::DomainResult;
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ModelUsageBreakdown, ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use async_trait::async_trait;
use chrono::{Datelike, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;
use walkdir::WalkDir;

pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        Self
    }

    fn resolve_dir(&self, account: &ProviderAccount, credentials: &ProviderCredentials) -> PathBuf {
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
            .join(".claude")
    }

    fn calculate_claude_cost(
        model: &str,
        input: u64,
        output: u64,
        cache_read: u64,
        cache_write: u64,
    ) -> f64 {
        let m = model.to_lowercase();
        let (in_p, out_p, cr_p, cw_p) = if m.contains("opus") {
            (15.0, 75.0, 1.5, 18.75)
        } else if m.contains("haiku") {
            (0.8, 4.0, 0.08, 1.0)
        } else {
            (3.0, 15.0, 0.3, 3.75)
        };
        (input as f64 / 1_000_000.0) * in_p
            + (output as f64 / 1_000_000.0) * out_p
            + (cache_read as f64 / 1_000_000.0) * cr_p
            + (cache_write as f64 / 1_000_000.0) * cw_p
    }
}

impl Default for ClaudeCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UsageProviderPort for ClaudeCodeAdapter {
    fn kind(&self) -> ProviderKind {
        ProviderKind::ClaudeCode
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let start = Instant::now();
        let root = self.resolve_dir(account, credentials);
        let projects_dir = if root.join("projects").exists() {
            root.join("projects")
        } else {
            root.clone()
        };

        let now = Utc::now();
        let current_month_prefix = format!("{:04}-{:02}", now.year(), now.month());
        let today_prefix = now.format("%Y-%m-%d").to_string();

        let mut total_input = 0_u64;
        let mut total_output = 0_u64;
        let mut total_cached = 0_u64;
        let mut total_requests = 0_u64;
        let mut daily_spend = 0.0_f64;
        let mut by_model: HashMap<String, (u64, u64, u64, f64)> = HashMap::new();

        if projects_dir.exists() {
            for entry in WalkDir::new(&projects_dir)
                .max_depth(5)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(path) else {
                    continue;
                };
                for line in content.lines() {
                    if !line.contains("usage") {
                        continue;
                    }
                    let Ok(val) = serde_json::from_str::<Value>(line) else {
                        continue;
                    };
                    let ts = val.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                    if !ts.is_empty() && !ts.starts_with(&current_month_prefix) {
                        continue;
                    }
                    let Some(usage) = val.pointer("/message/usage") else {
                        continue;
                    };
                    let input = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let cache_read = usage
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let cache_write = usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if input == 0 && output == 0 && cache_read == 0 {
                        continue;
                    }
                    let model = val
                        .pointer("/message/model")
                        .and_then(|v| v.as_str())
                        .unwrap_or("claude-sonnet-4")
                        .to_string();
                    let cost =
                        Self::calculate_claude_cost(&model, input, output, cache_read, cache_write);

                    total_input = total_input.saturating_add(input);
                    total_output = total_output.saturating_add(output);
                    total_cached = total_cached
                        .saturating_add(cache_read)
                        .saturating_add(cache_write);
                    total_requests = total_requests.saturating_add(1);
                    if ts.starts_with(&today_prefix) {
                        daily_spend += cost;
                    }
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
        let limit = account.monthly_budget_usd.unwrap_or(100.0);
        let util = ((spend_usd / limit) * 100.0).clamp(0.0, 100.0);

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: ProviderKind::ClaudeCode,
            recorded_at: Utc::now(),
            status: SyncStatus::Healthy,
            spend_usd,
            daily_spend_usd: daily_spend,
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
                label: "Monthly Claude Code Sessions".to_string(),
                used: spend_usd,
                limit,
                unit: "USD".to_string(),
                resets_at: None,
            }],
            model_breakdown: breakdown,
            latency_ms: start.elapsed().as_millis() as u64,
            error_message: None,
            source_note: format!("~/.claude JSONL · {total_requests} calls"),
        })
    }
}
