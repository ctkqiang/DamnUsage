use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Antigravity,
    ClaudeCode,
    DeepSeek,
    OpenRouter,
    AmazonBedrock,
    OpenAi,
    AnthropicApi,
    GoogleGemini,
    XaiGrok,
    SiliconFlow,
    MoonshotKimi,
    ZhipuGlm,
    QwenDashScope,
    Groq,
    TogetherAi,
    MistralAi,
    FireworksAi,
    PerplexityAi,
    CursorIde,
    GithubCopilot,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Antigravity => "antigravity",
            Self::ClaudeCode => "claude_code",
            Self::DeepSeek => "deep_seek",
            Self::OpenRouter => "open_router",
            Self::AmazonBedrock => "amazon_bedrock",
            Self::OpenAi => "open_ai",
            Self::AnthropicApi => "anthropic_api",
            Self::GoogleGemini => "google_gemini",
            Self::XaiGrok => "xai_grok",
            Self::SiliconFlow => "silicon_flow",
            Self::MoonshotKimi => "moonshot_kimi",
            Self::ZhipuGlm => "zhipu_glm",
            Self::QwenDashScope => "qwen_dash_scope",
            Self::Groq => "groq",
            Self::TogetherAi => "together_ai",
            Self::MistralAi => "mistral_ai",
            Self::FireworksAi => "fireworks_ai",
            Self::PerplexityAi => "perplexity_ai",
            Self::CursorIde => "cursor_ide",
            Self::GithubCopilot => "github_copilot",
        }
    }

    pub fn all() -> &'static [ProviderKind] {
        &[
            Self::Antigravity,
            Self::ClaudeCode,
            Self::DeepSeek,
            Self::OpenRouter,
            Self::AmazonBedrock,
            Self::OpenAi,
            Self::AnthropicApi,
            Self::GoogleGemini,
            Self::XaiGrok,
            Self::SiliconFlow,
            Self::MoonshotKimi,
            Self::ZhipuGlm,
            Self::QwenDashScope,
            Self::Groq,
            Self::TogetherAi,
            Self::MistralAi,
            Self::FireworksAi,
            Self::PerplexityAi,
            Self::CursorIde,
            Self::GithubCopilot,
        ]
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCategory {
    CodingAgent,
    CloudPlatform,
    Aggregator,
    GlobalApi,
    ChineseApi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthStrategy {
    LocalAuto,
    ApiKey,
    AdminApiKey,
    AwsSigV4,
    OAuthToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocaleCode {
    En,
    ZhCn,
    ZhTw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum CurrencyCode {
    Usd,
    Cny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotchDisplayState {
    Collapsed,
    Peek,
    Expanded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Healthy,
    Syncing,
    Warning,
    Critical,
    Error,
    Simulated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialFieldSpec {
    pub key: String,
    pub label_key: String,
    pub placeholder: String,
    pub secret: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCatalogEntry {
    pub kind: ProviderKind,
    pub display_name: String,
    pub short_name: String,
    pub category: ProviderCategory,
    pub auth_strategy: AuthStrategy,
    pub accent_hex: String,
    pub secondary_hex: String,
    pub default_endpoint: String,
    pub docs_url: String,
    pub supports_balance: bool,
    pub supports_token_breakdown: bool,
    pub credential_fields: Vec<CredentialFieldSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCredentials {
    pub fields: HashMap<String, String>,
}

impl ProviderCredentials {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .get(key)
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub total_requests: u64,
}

impl TokenMetrics {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cached_tokens)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageBreakdown {
    pub model_id: String,
    pub spend_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaWindow {
    pub label: String,
    pub used: f64,
    pub limit: f64,
    pub unit: String,
    pub resets_at: Option<String>,
}

impl QuotaWindow {
    pub fn utilization_pct(&self) -> f64 {
        if self.limit <= 0.0 {
            return 0.0;
        }
        ((self.used / self.limit) * 100.0).clamp(0.0, 100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparklinePoint {
    pub timestamp: String,
    pub spend_usd: f64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAlert {
    pub id: String,
    pub provider_id: String,
    pub provider_name: String,
    pub severity: AlertSeverity,
    pub message_key: String,
    pub utilization_pct: f64,
    pub triggered_at: String,
}
