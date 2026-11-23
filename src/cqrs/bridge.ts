import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConfigureProviderCommand,
  DashboardReadModel,
  NotchDisplayState,
  NotchGeometry,
  ProviderAccount,
  ProviderCatalogEntry,
  ProviderCatalogItemView,
  ProviderDetailReadModel,
  ProviderKind,
  ProviderSummaryReadModel,
  SparklinePoint,
  UpdatePreferencesCommand,
  UsageSnapshot,
  UserPreferences,
} from "./types";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
}

const FALLBACK_CATALOG: ProviderCatalogEntry[] = [
  {
    kind: "antigravity",
    display_name: "Antigravity (AGY)",
    short_name: "AGY",
    category: "coding_agent",
    auth_strategy: "local_auto",
    accent_hex: "#7C5CFF",
    secondary_hex: "#00D4FF",
    default_endpoint: "~/.gemini/antigravity-cli",
    docs_url: "https://antigravity.google",
    supports_balance: true,
    supports_token_breakdown: true,
    credential_fields: [
      { key: "data_dir", label_key: "fields.data_dir", placeholder: "~/.gemini/antigravity-cli", secret: false, required: false },
      { key: "api_key", label_key: "fields.optional_api_key", placeholder: "agy_...", secret: true, required: false },
    ],
  },
  {
    kind: "claude_code",
    display_name: "Claude Code",
    short_name: "Claude",
    category: "coding_agent",
    auth_strategy: "local_auto",
    accent_hex: "#D97757",
    secondary_hex: "#F4A261",
    default_endpoint: "~/.claude/projects",
    docs_url: "https://docs.anthropic.com",
    supports_balance: true,
    supports_token_breakdown: true,
    credential_fields: [
      { key: "data_dir", label_key: "fields.claude_dir", placeholder: "~/.claude", secret: false, required: false },
      { key: "admin_key", label_key: "fields.anthropic_admin_key", placeholder: "sk-ant-admin-...", secret: true, required: false },
    ],
  },
  {
    kind: "deep_seek",
    display_name: "DeepSeek",
    short_name: "DeepSeek",
    category: "chinese_api",
    auth_strategy: "api_key",
    accent_hex: "#4D6BFE",
    secondary_hex: "#38BDF8",
    default_endpoint: "https://api.deepseek.com",
    docs_url: "https://api-docs.deepseek.com",
    supports_balance: true,
    supports_token_breakdown: false,
    credential_fields: [
      { key: "api_key", label_key: "fields.api_key", placeholder: "sk-...", secret: true, required: true },
    ],
  },
  {
    kind: "open_router",
    display_name: "OpenRouter",
    short_name: "OpenRouter",
    category: "aggregator",
    auth_strategy: "api_key",
    accent_hex: "#6366F1",
    secondary_hex: "#A855F7",
    default_endpoint: "https://openrouter.ai/api/v1",
    docs_url: "https://openrouter.ai",
    supports_balance: true,
    supports_token_breakdown: true,
    credential_fields: [
      { key: "api_key", label_key: "fields.api_key", placeholder: "sk-or-v1-...", secret: true, required: true },
    ],
  },
  {
    kind: "amazon_bedrock",
    display_name: "Amazon Bedrock",
    short_name: "Bedrock",
    category: "cloud_platform",
    auth_strategy: "aws_sig_v4",
    accent_hex: "#FF9900",
    secondary_hex: "#FFB84D",
    default_endpoint: "https://ce.us-east-1.amazonaws.com",
    docs_url: "https://docs.aws.amazon.com/bedrock",
    supports_balance: false,
    supports_token_breakdown: true,
    credential_fields: [
      { key: "aws_access_key_id", label_key: "fields.aws_access_key", placeholder: "AKIA...", secret: true, required: true },
      { key: "aws_secret_access_key", label_key: "fields.aws_secret_key", placeholder: "Secret Key", secret: true, required: true },
      { key: "aws_session_token", label_key: "fields.aws_session_token", placeholder: "Optional STS", secret: true, required: false },
      { key: "aws_region", label_key: "fields.aws_region", placeholder: "us-east-1", secret: false, required: true },
    ],
  },
  {
    kind: "open_ai",
    display_name: "OpenAI",
    short_name: "OpenAI",
    category: "global_api",
    auth_strategy: "admin_api_key",
    accent_hex: "#10A37F",
    secondary_hex: "#34D399",
    default_endpoint: "https://api.openai.com/v1",
    docs_url: "https://platform.openai.com",
    supports_balance: false,
    supports_token_breakdown: true,
    credential_fields: [
      { key: "api_key", label_key: "fields.openai_admin_key", placeholder: "sk-admin-...", secret: true, required: true },
    ],
  },
  {
    kind: "moonshot_kimi",
    display_name: "Moonshot Kimi",
    short_name: "Kimi",
    category: "chinese_api",
    auth_strategy: "api_key",
    accent_hex: "#0EA5E9",
    secondary_hex: "#22D3EE",
    default_endpoint: "https://api.moonshot.cn/v1",
    docs_url: "https://platform.moonshot.cn",
    supports_balance: true,
    supports_token_breakdown: false,
    credential_fields: [
      { key: "api_key", label_key: "fields.api_key", placeholder: "sk-...", secret: true, required: true },
    ],
  },
  {
    kind: "silicon_flow",
    display_name: "SiliconFlow",
    short_name: "Silicon",
    category: "aggregator",
    auth_strategy: "api_key",
    accent_hex: "#8B5CF6",
    secondary_hex: "#EC4899",
    default_endpoint: "https://api.siliconflow.cn/v1",
    docs_url: "https://docs.siliconflow.cn",
    supports_balance: true,
    supports_token_breakdown: false,
    credential_fields: [
      { key: "api_key", label_key: "fields.api_key", placeholder: "sk-...", secret: true, required: true },
    ],
  },
];

function makeSparkline(base: number): SparklinePoint[] {
  const now = Date.now();
  return Array.from({ length: 12 }, (_, idx) => {
    const ratio = 0.65 + (idx / 11) * 0.35 + Math.sin(idx * 0.9) * 0.04;
    return {
      timestamp: new Date(now - (11 - idx) * 3600_000).toISOString(),
      spend_usd: Number((base * ratio).toFixed(2)),
      tokens: Math.round(1_200_000 * ratio),
    };
  });
}

class BrowserCqrsFallback {
  private prefs: UserPreferences = {
    locale: "en",
    currency: "USD",
    usd_to_cny_rate: 7.24,
    refresh_interval_secs: 60,
    monthly_global_budget_usd: 420,
    warning_threshold_pct: 75,
    critical_threshold_pct: 90,
    demo_mode: true,
    hover_to_peek: true,
    show_cost_in_notch: true,
    notch_width_offset_px: 0,
  };

  private notchState: NotchDisplayState = "expanded";

  private providers: ProviderSummaryReadModel[] = [
    {
      provider_id: "demo-antigravity",
      kind: "antigravity",
      label: "Antigravity (AGY)",
      short_name: "AGY",
      category: "coding_agent",
      accent_hex: "#7C5CFF",
      secondary_hex: "#00D4FF",
      status: "simulated",
      spend_usd: 38.42,
      daily_spend_usd: 3.18,
      balance_usd: 21.58,
      monthly_limit_usd: 60,
      utilization_pct: 64.0,
      total_tokens: 18_760_000,
      total_requests: 512,
      last_updated: new Date().toISOString(),
      source_note: "live_preview",
      sparkline: makeSparkline(38.42),
    },
    {
      provider_id: "demo-claude_code",
      kind: "claude_code",
      label: "Claude Code",
      short_name: "Claude",
      category: "coding_agent",
      accent_hex: "#D97757",
      secondary_hex: "#F4A261",
      status: "warning",
      spend_usd: 76.85,
      daily_spend_usd: 6.42,
      balance_usd: 23.15,
      monthly_limit_usd: 100,
      utilization_pct: 76.85,
      total_tokens: 24_260_000,
      total_requests: 642,
      last_updated: new Date().toISOString(),
      source_note: "live_preview",
      sparkline: makeSparkline(76.85),
    },
    {
      provider_id: "demo-amazon_bedrock",
      kind: "amazon_bedrock",
      label: "Amazon Bedrock",
      short_name: "Bedrock",
      category: "cloud_platform",
      accent_hex: "#FF9900",
      secondary_hex: "#FFB84D",
      status: "simulated",
      spend_usd: 89.3,
      daily_spend_usd: 7.65,
      balance_usd: 30.7,
      monthly_limit_usd: 120,
      utilization_pct: 74.4,
      total_tokens: 28_020_000,
      total_requests: 814,
      last_updated: new Date().toISOString(),
      source_note: "AWS SigV4 · us-east-1",
      sparkline: makeSparkline(89.3),
    },
    {
      provider_id: "demo-open_router",
      kind: "open_router",
      label: "OpenRouter",
      short_name: "OpenRouter",
      category: "aggregator",
      accent_hex: "#6366F1",
      secondary_hex: "#A855F7",
      status: "simulated",
      spend_usd: 44.2,
      daily_spend_usd: 4.15,
      balance_usd: 35.8,
      monthly_limit_usd: 80,
      utilization_pct: 55.25,
      total_tokens: 14_310_000,
      total_requests: 498,
      last_updated: new Date().toISOString(),
      source_note: "openrouter.ai/api/v1/auth/key",
      sparkline: makeSparkline(44.2),
    },
    {
      provider_id: "demo-deep_seek",
      kind: "deep_seek",
      label: "DeepSeek",
      short_name: "DeepSeek",
      category: "chinese_api",
      accent_hex: "#4D6BFE",
      secondary_hex: "#38BDF8",
      status: "simulated",
      spend_usd: 12.64,
      daily_spend_usd: 1.08,
      balance_usd: 17.36,
      monthly_limit_usd: 30,
      utilization_pct: 42.1,
      total_tokens: 37_350_000,
      total_requests: 1290,
      last_updated: new Date().toISOString(),
      source_note: "api.deepseek.com/user/balance",
      sparkline: makeSparkline(12.64),
    },
  ];

  private listeners: Array<() => void> = [];

  subscribe(cb: () => void): UnlistenFn {
    this.listeners.push(cb);
    return () => {
      this.listeners = this.listeners.filter((fn) => fn !== cb);
    };
  }

  private emit() {
    this.listeners.forEach((fn) => fn());
  }

  getDashboard(): DashboardReadModel {
    const totalSpend = this.providers.reduce((sum, p) => sum + p.spend_usd, 0);
    const totalDaily = this.providers.reduce((sum, p) => sum + p.daily_spend_usd, 0);
    const totalTokens = this.providers.reduce((sum, p) => sum + p.total_tokens, 0);
    const totalRequests = this.providers.reduce((sum, p) => sum + p.total_requests, 0);
    const util = Math.min(100, (totalSpend / this.prefs.monthly_global_budget_usd) * 100);
    const lead = this.providers[0];

    return {
      generated_at: new Date().toISOString(),
      last_synced_at: new Date().toISOString(),
      notch_state: this.notchState,
      demo_mode: this.prefs.demo_mode,
      currency: this.prefs.currency,
      usd_to_cny_rate: this.prefs.usd_to_cny_rate,
      monthly_global_budget_usd: this.prefs.monthly_global_budget_usd,
      total_spend_usd: totalSpend,
      total_spend_cny: totalSpend * this.prefs.usd_to_cny_rate,
      daily_burn_rate_usd: totalDaily,
      total_tokens: totalTokens,
      total_requests: totalRequests,
      global_utilization_pct: util,
      headline: {
        primary_provider_name: lead?.short_name ?? "DamnUsage",
        primary_accent_hex: lead?.accent_hex ?? "#7C5CFF",
        total_spend_usd: totalSpend,
        total_spend_cny: totalSpend * this.prefs.usd_to_cny_rate,
        global_utilization_pct: util,
        active_provider_count: this.providers.length,
        alert_count: this.providers.filter((p) => p.utilization_pct >= this.prefs.warning_threshold_pct).length,
        status: "simulated",
      },
      providers: [...this.providers].sort((a, b) => b.spend_usd - a.spend_usd),
      alerts: [],
      aggregate_sparkline: makeSparkline(totalSpend),
    };
  }

  getCatalog(): ProviderCatalogItemView[] {
    return FALLBACK_CATALOG.map((entry) => {
      const ids = this.providers
        .filter((p) => p.kind === entry.kind)
        .map((p) => p.provider_id);
      return {
        entry,
        configured_count: ids.length,
        configured_ids: ids,
      };
    });
  }

  getPreferences(): UserPreferences {
    return { ...this.prefs };
  }

  getProviderDetail(providerId: string): ProviderDetailReadModel {
    const summary = this.providers.find((p) => p.provider_id === providerId) ?? this.providers[0];
    const catalog =
      FALLBACK_CATALOG.find((c) => c.kind === summary.kind) ?? FALLBACK_CATALOG[0];
    return {
      account: {
        id: summary.provider_id,
        kind: summary.kind,
        label: summary.label,
        enabled: true,
        monthly_budget_usd: summary.monthly_limit_usd,
        endpoint_override: null,
        metadata: {},
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
      catalog,
      snapshot: {
        snapshot_id: "preview",
        provider_id: summary.provider_id,
        kind: summary.kind,
        recorded_at: new Date().toISOString(),
        status: summary.status,
        spend_usd: summary.spend_usd,
        daily_spend_usd: summary.daily_spend_usd,
        balance_usd: summary.balance_usd,
        monthly_limit_usd: summary.monthly_limit_usd,
        utilization_pct: summary.utilization_pct,
        tokens: {
          input_tokens: Math.round(summary.total_tokens * 0.72),
          output_tokens: Math.round(summary.total_tokens * 0.18),
          cached_tokens: Math.round(summary.total_tokens * 0.1),
          total_requests: summary.total_requests,
        },
        quota_windows: [
          {
            label: "Monthly Budget",
            used: summary.spend_usd,
            limit: summary.monthly_limit_usd ?? 100,
            unit: "USD",
            resets_at: null,
          },
        ],
        model_breakdown: [
          {
            model_id: `${summary.short_name.toLowerCase()}-flagship`,
            spend_usd: summary.spend_usd * 0.68,
            input_tokens: Math.round(summary.total_tokens * 0.5),
            output_tokens: Math.round(summary.total_tokens * 0.12),
            request_count: Math.round(summary.total_requests * 0.6),
          },
          {
            model_id: `${summary.short_name.toLowerCase()}-fast`,
            spend_usd: summary.spend_usd * 0.32,
            input_tokens: Math.round(summary.total_tokens * 0.22),
            output_tokens: Math.round(summary.total_tokens * 0.06),
            request_count: Math.round(summary.total_requests * 0.4),
          },
        ],
        latency_ms: 54,
        error_message: null,
        source_note: summary.source_note,
      },
      history: summary.sparkline,
    };
  }

  updatePreferences(cmd: UpdatePreferencesCommand): UserPreferences {
    this.prefs = { ...this.prefs, ...cmd };
    this.emit();
    return this.getPreferences();
  }

  transitionNotch(target: NotchDisplayState): NotchDisplayState {
    this.notchState = target;
    this.emit();
    return target;
  }

  configureProvider(cmd: ConfigureProviderCommand): ProviderAccount {
    const entry =
      FALLBACK_CATALOG.find((c) => c.kind === cmd.kind) ?? FALLBACK_CATALOG[0];
    const id = cmd.provider_id ?? `local-${cmd.kind}-${Date.now()}`;
    const existingIdx = this.providers.findIndex((p) => p.provider_id === id);
    const budget = cmd.monthly_budget_usd ?? 50;
    const item: ProviderSummaryReadModel = {
      provider_id: id,
      kind: cmd.kind as ProviderKind,
      label: cmd.label || entry.display_name,
      short_name: entry.short_name,
      category: entry.category,
      accent_hex: entry.accent_hex,
      secondary_hex: entry.secondary_hex,
      status: "healthy",
      spend_usd: 14.2,
      daily_spend_usd: 1.35,
      balance_usd: Math.max(0, budget - 14.2),
      monthly_limit_usd: budget,
      utilization_pct: Math.min(100, (14.2 / budget) * 100),
      total_tokens: 4_200_000,
      total_requests: 190,
      last_updated: new Date().toISOString(),
      source_note: entry.default_endpoint,
      sparkline: makeSparkline(14.2),
    };
    if (existingIdx >= 0) {
      this.providers[existingIdx] = item;
    } else {
      this.providers.push(item);
    }
    this.emit();
    return {
      id,
      kind: cmd.kind,
      label: item.label,
      enabled: cmd.enabled,
      monthly_budget_usd: budget,
      endpoint_override: cmd.endpoint_override,
      metadata: cmd.metadata,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
  }

  removeProvider(providerId: string): void {
    this.providers = this.providers.filter((p) => p.provider_id !== providerId);
    this.emit();
  }

  syncAll(): UsageSnapshot[] {
    this.providers = this.providers.map((p) => {
      const bump = Number((Math.random() * 0.25).toFixed(2));
      const spend = Number((p.spend_usd + bump).toFixed(2));
      const lim = p.monthly_limit_usd ?? 100;
      return {
        ...p,
        spend_usd: spend,
        utilization_pct: Math.min(100, (spend / lim) * 100),
        total_tokens: p.total_tokens + 24_500,
        last_updated: new Date().toISOString(),
      };
    });
    this.emit();
    return [];
  }
}

export const browserFallback = new BrowserCqrsFallback();

export async function invokeTauri<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

export async function subscribeDomainEvents(onEvent: () => void): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    return browserFallback.subscribe(onEvent);
  }
  return listen("damnusage://domain-event", () => {
    onEvent();
  });
}

export async function fetchNotchGeometry(): Promise<NotchGeometry> {
  if (!isTauriRuntime()) {
    return {
      has_hardware_notch: true,
      notch_height_px: 34,
      screen_width_px: 1512,
      scale_factor: 2,
    };
  }
  return invokeTauri<NotchGeometry>("qry_notch_geometry");
}
