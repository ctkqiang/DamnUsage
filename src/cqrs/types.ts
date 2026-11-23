export type ProviderKind =
  | "antigravity"
  | "claude_code"
  | "deep_seek"
  | "open_router"
  | "amazon_bedrock"
  | "open_ai"
  | "anthropic_api"
  | "google_gemini"
  | "xai_grok"
  | "silicon_flow"
  | "moonshot_kimi"
  | "zhipu_glm"
  | "qwen_dash_scope"
  | "groq"
  | "together_ai"
  | "mistral_ai"
  | "fireworks_ai"
  | "perplexity_ai"
  | "cursor_ide"
  | "github_copilot";

export type ProviderCategory =
  | "coding_agent"
  | "cloud_platform"
  | "aggregator"
  | "global_api"
  | "chinese_api";

export type AuthStrategy =
  | "local_auto"
  | "api_key"
  | "admin_api_key"
  | "aws_sig_v4"
  | "oauth_token";

export type LocaleCode = "en" | "zh_cn" | "zh_tw";
export type CurrencyCode = "USD" | "CNY";
export type NotchDisplayState = "collapsed" | "peek" | "expanded";
export type SyncStatus =
  | "healthy"
  | "syncing"
  | "warning"
  | "critical"
  | "error"
  | "simulated";
export type AlertSeverity = "info" | "warning" | "critical";

export interface CredentialFieldSpec {
  key: string;
  label_key: string;
  placeholder: string;
  secret: boolean;
  required: boolean;
}

export interface ProviderCatalogEntry {
  kind: ProviderKind;
  display_name: string;
  short_name: string;
  category: ProviderCategory;
  auth_strategy: AuthStrategy;
  accent_hex: string;
  secondary_hex: string;
  default_endpoint: string;
  docs_url: string;
  supports_balance: boolean;
  supports_token_breakdown: boolean;
  credential_fields: CredentialFieldSpec[];
}

export interface ProviderCatalogItemView {
  entry: ProviderCatalogEntry;
  configured_count: number;
  configured_ids: string[];
}

export interface SparklinePoint {
  timestamp: string;
  spend_usd: number;
  tokens: number;
}

export interface UsageAlert {
  id: string;
  provider_id: string;
  provider_name: string;
  severity: AlertSeverity;
  message_key: string;
  utilization_pct: number;
  triggered_at: string;
}

export interface NotchHeadlineReadModel {
  primary_provider_name: string;
  primary_accent_hex: string;
  total_spend_usd: number;
  total_spend_cny: number;
  global_utilization_pct: number;
  active_provider_count: number;
  alert_count: number;
  status: SyncStatus;
}

export interface ProviderSummaryReadModel {
  provider_id: string;
  kind: ProviderKind;
  label: string;
  short_name: string;
  category: ProviderCategory;
  accent_hex: string;
  secondary_hex: string;
  status: SyncStatus;
  spend_usd: number;
  daily_spend_usd: number;
  balance_usd: number | null;
  monthly_limit_usd: number | null;
  utilization_pct: number;
  total_tokens: number;
  total_requests: number;
  last_updated: string | null;
  source_note: string;
  sparkline: SparklinePoint[];
}

export interface DashboardReadModel {
  generated_at: string;
  last_synced_at: string | null;
  notch_state: NotchDisplayState;
  demo_mode: boolean;
  currency: CurrencyCode;
  usd_to_cny_rate: number;
  monthly_global_budget_usd: number;
  total_spend_usd: number;
  total_spend_cny: number;
  daily_burn_rate_usd: number;
  total_tokens: number;
  total_requests: number;
  global_utilization_pct: number;
  headline: NotchHeadlineReadModel;
  providers: ProviderSummaryReadModel[];
  alerts: UsageAlert[];
  aggregate_sparkline: SparklinePoint[];
}

export interface TokenMetrics {
  input_tokens: number;
  output_tokens: number;
  cached_tokens: number;
  total_requests: number;
}

export interface QuotaWindow {
  label: string;
  used: number;
  limit: number;
  unit: string;
  resets_at: string | null;
}

export interface ModelUsageBreakdown {
  model_id: string;
  spend_usd: number;
  input_tokens: number;
  output_tokens: number;
  request_count: number;
}

export interface UsageSnapshot {
  snapshot_id: string;
  provider_id: string;
  kind: ProviderKind;
  recorded_at: string;
  status: SyncStatus;
  spend_usd: number;
  daily_spend_usd: number;
  balance_usd: number | null;
  monthly_limit_usd: number | null;
  utilization_pct: number;
  tokens: TokenMetrics;
  quota_windows: QuotaWindow[];
  model_breakdown: ModelUsageBreakdown[];
  latency_ms: number;
  error_message: string | null;
  source_note: string;
}

export interface ProviderAccount {
  id: string;
  kind: ProviderKind;
  label: string;
  enabled: boolean;
  monthly_budget_usd: number | null;
  endpoint_override: string | null;
  metadata: Record<string, string>;
  created_at: string;
  updated_at: string;
}

export interface ProviderDetailReadModel {
  account: ProviderAccount;
  catalog: ProviderCatalogEntry;
  snapshot: UsageSnapshot | null;
  history: SparklinePoint[];
}

export interface UserPreferences {
  locale: LocaleCode;
  currency: CurrencyCode;
  usd_to_cny_rate: number;
  refresh_interval_secs: number;
  monthly_global_budget_usd: number;
  warning_threshold_pct: number;
  critical_threshold_pct: number;
  demo_mode: boolean;
  hover_to_peek: boolean;
  show_cost_in_notch: boolean;
  notch_width_offset_px: number;
}

export interface NotchGeometry {
  has_hardware_notch: boolean;
  notch_height_px: number;
  screen_width_px: number;
  scale_factor: number;
}

export interface ConfigureProviderCommand {
  provider_id: string | null;
  kind: ProviderKind;
  label: string;
  enabled: boolean;
  monthly_budget_usd: number | null;
  endpoint_override: string | null;
  credentials: Record<string, string>;
  metadata: Record<string, string>;
}

export interface UpdatePreferencesCommand {
  locale?: LocaleCode;
  currency?: CurrencyCode;
  usd_to_cny_rate?: number;
  refresh_interval_secs?: number;
  monthly_global_budget_usd?: number;
  warning_threshold_pct?: number;
  critical_threshold_pct?: number;
  demo_mode?: boolean;
  hover_to_peek?: boolean;
  show_cost_in_notch?: boolean;
  notch_width_offset_px?: number;
}
