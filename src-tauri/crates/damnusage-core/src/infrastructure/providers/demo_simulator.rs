use crate::domain::entities::{ProviderAccount, UsageSnapshot};
use crate::domain::errors::DomainResult;
use crate::domain::ports::UsageProviderPort;
use crate::domain::value_objects::{
    ModelUsageBreakdown, ProviderCredentials, ProviderKind, QuotaWindow, SyncStatus, TokenMetrics,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

pub struct DemoUsageSimulator {
    tick: AtomicU64,
}

impl DemoUsageSimulator {
    pub fn new() -> Self {
        Self {
            tick: AtomicU64::new(0),
        }
    }

    fn base_profile(kind: ProviderKind) -> (f64, f64, f64, u64, u64, Vec<(&'static str, f64)>) {
        match kind {
            ProviderKind::Antigravity => (
                38.42,
                3.18,
                60.0,
                14_820_000,
                3_940_000,
                vec![("gemini-2.5-pro", 26.8), ("gemini-2.5-flash", 11.62)],
            ),
            ProviderKind::ClaudeCode => (
                76.85,
                6.42,
                100.0,
                19_450_000,
                4_810_000,
                vec![
                    ("claude-sonnet-4-20250514", 49.1),
                    ("claude-opus-4-20250514", 27.75),
                ],
            ),
            ProviderKind::OpenRouter => (
                44.20,
                4.15,
                80.0,
                11_200_000,
                3_110_000,
                vec![
                    ("anthropic/claude-3.7-sonnet", 21.4),
                    ("deepseek/deepseek-r1", 12.9),
                    ("openai/o3-mini", 9.9),
                ],
            ),
            ProviderKind::DeepSeek => (
                12.64,
                1.08,
                30.0,
                28_900_000,
                8_450_000,
                vec![("deepseek-reasoner", 8.14), ("deepseek-chat", 4.50)],
            ),
            ProviderKind::AmazonBedrock => (
                89.30,
                7.65,
                120.0,
                22_600_000,
                5_420_000,
                vec![
                    ("anthropic.claude-3-7-sonnet-v1", 58.2),
                    ("amazon.nova-pro-v1", 19.4),
                    ("meta.llama3-3-70b", 11.7),
                ],
            ),
            ProviderKind::OpenAi => (
                41.15,
                3.45,
                75.0,
                9_800_000,
                2_640_000,
                vec![("gpt-4.1", 24.3), ("o3", 16.85)],
            ),
            ProviderKind::MoonshotKimi => (
                9.82,
                0.94,
                25.0,
                16_400_000,
                4_120_000,
                vec![("kimi-k2-0711-preview", 9.82)],
            ),
            ProviderKind::SiliconFlow => (
                7.45,
                0.71,
                20.0,
                18_200_000,
                5_300_000,
                vec![
                    ("Pro/deepseek-ai/DeepSeek-R1", 4.95),
                    ("Qwen/Qwen2.5-Coder-32B", 2.50),
                ],
            ),
            _ => (
                18.50,
                1.65,
                50.0,
                6_200_000,
                1_800_000,
                vec![("flagship-model", 18.50)],
            ),
        }
    }
}

impl Default for DemoUsageSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UsageProviderPort for DemoUsageSimulator {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Antigravity
    }

    async fn fetch_usage(
        &self,
        account: &ProviderAccount,
        _credentials: &ProviderCredentials,
    ) -> DomainResult<UsageSnapshot> {
        let step = self.tick.fetch_add(1, Ordering::Relaxed);
        let (base_spend, base_daily, default_limit, base_in, base_out, models) =
            Self::base_profile(account.kind);

        let jitter = ((step % 17) as f64) * 0.07;
        let spend_usd = base_spend + jitter;
        let daily_spend_usd = base_daily + (jitter * 0.35);
        let limit = account.monthly_budget_usd.unwrap_or(default_limit);
        let utilization_pct = ((spend_usd / limit.max(1.0)) * 100.0).clamp(0.0, 100.0);
        let balance_usd = Some((limit - spend_usd).max(0.0));

        let in_tokens = base_in + step * 18_420;
        let out_tokens = base_out + step * 4_910;
        let cached_tokens = in_tokens / 3;
        let requests = 420 + step * 3;

        let breakdown = models
            .into_iter()
            .enumerate()
            .map(|(i, (name, share))| ModelUsageBreakdown {
                model_id: name.to_string(),
                spend_usd: share + (jitter / (i as f64 + 1.0)),
                input_tokens: in_tokens / (i as u64 + 2),
                output_tokens: out_tokens / (i as u64 + 2),
                request_count: requests / (i as u64 + 2),
            })
            .collect();

        let reset_at = (Utc::now() + Duration::hours(5)).to_rfc3339();

        Ok(UsageSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            provider_id: account.id.clone(),
            kind: account.kind,
            recorded_at: Utc::now(),
            status: SyncStatus::Simulated,
            spend_usd,
            daily_spend_usd,
            balance_usd,
            monthly_limit_usd: Some(limit),
            utilization_pct,
            tokens: TokenMetrics {
                input_tokens: in_tokens,
                output_tokens: out_tokens,
                cached_tokens,
                total_requests: requests,
            },
            quota_windows: vec![
                QuotaWindow {
                    label: "Monthly Budget".to_string(),
                    used: spend_usd,
                    limit,
                    unit: "USD".to_string(),
                    resets_at: None,
                },
                QuotaWindow {
                    label: "5h Rolling Window".to_string(),
                    used: (utilization_pct * 0.82).min(98.0),
                    limit: 100.0,
                    unit: "%".to_string(),
                    resets_at: Some(reset_at),
                },
            ],
            model_breakdown: breakdown,
            latency_ms: 42 + (step % 38),
            error_message: None,
            source_note: "live_preview".to_string(),
        })
    }
}
