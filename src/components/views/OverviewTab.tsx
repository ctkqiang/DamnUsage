import React from "react";
import { motion } from "framer-motion";
import { Activity, AlertTriangle, ChevronRight, Cpu, Flame, Layers } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { DashboardReadModel } from "../../cqrs/types";
import { formatCompactNumber, formatMoney } from "../../utils/format";
import { AnimatedNumber } from "../visualizations/AnimatedNumber";
import { LiquidQuotaBar } from "../visualizations/LiquidQuotaBar";
import { ProviderGlyph } from "../visualizations/ProviderGlyph";
import { RadialGauge } from "../visualizations/RadialGauge";
import { SparklineChart } from "../visualizations/SparklineChart";

interface OverviewTabProps {
  dashboard: DashboardReadModel;
  onSelectProvider: (providerId: string) => void;
  onOpenProviders: () => void;
}

export const OverviewTab: React.FC<OverviewTabProps> = ({
  dashboard,
  onSelectProvider,
  onOpenProviders,
}) => {
  const { t } = useTranslation();
  const prefix = dashboard.currency === "CNY" ? "¥" : "$";
  const heroSpend =
    dashboard.currency === "CNY" ? dashboard.total_spend_cny : dashboard.total_spend_usd;
  const heroBudget =
    dashboard.currency === "CNY"
      ? dashboard.monthly_global_budget_usd * dashboard.usd_to_cny_rate
      : dashboard.monthly_global_budget_usd;

  return (
    <motion.div
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -6 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col gap-3"
    >
      <div className="grid grid-cols-12 gap-2.5">
        <div className="notch-card col-span-7 flex items-center justify-between rounded-2xl p-3.5">
          <div className="flex flex-col gap-1">
            <span className="text-[10px] font-medium uppercase tracking-wider text-white/45">
              {t("overview.monthlySpend")}
            </span>
            <div className="flex items-baseline gap-1.5">
              <AnimatedNumber
                value={heroSpend}
                prefix={prefix}
                decimals={2}
                className="font-mono text-2xl font-bold tracking-tight text-white"
              />
              <span className="font-mono text-[11px] text-white/40">
                / {prefix}
                {heroBudget.toFixed(0)}
              </span>
            </div>
            <div className="mt-1 flex items-center gap-3 text-[10px] text-white/60">
              <span className="inline-flex items-center gap-1">
                <Flame className="h-3 w-3 text-amber-400" />
                {t("overview.dailyBurn")}:{" "}
                <strong className="font-mono text-white/90">
                  {formatMoney(
                    dashboard.daily_burn_rate_usd,
                    dashboard.currency,
                    dashboard.usd_to_cny_rate,
                  )}
                </strong>
              </span>
              <span className="inline-flex items-center gap-1">
                <Cpu className="h-3 w-3 text-cyan-400" />
                <strong className="font-mono text-white/90">
                  {formatCompactNumber(dashboard.total_tokens)}
                </strong>
              </span>
            </div>
          </div>
          <RadialGauge
            percent={dashboard.global_utilization_pct}
            size={62}
            strokeWidth={5.5}
            accentHex={dashboard.headline.primary_accent_hex}
            secondaryHex="#00D4FF"
          />
        </div>

        <div className="notch-card col-span-5 flex flex-col justify-between rounded-2xl p-3.5">
          <div className="flex items-center justify-between">
            <span className="inline-flex items-center gap-1 text-[10px] font-medium uppercase tracking-wider text-white/45">
              <Activity className="h-3 w-3 text-violet-400" />
              {t("overview.trend24h")}
            </span>
            <span className="font-mono text-[10px] text-white/55">
              {formatCompactNumber(dashboard.total_requests)} {t("overview.requests")}
            </span>
          </div>
          <div className="mt-1 flex items-end justify-center">
            <SparklineChart
              points={dashboard.aggregate_sparkline}
              accentHex={dashboard.headline.primary_accent_hex}
              secondaryHex="#00D4FF"
              width={215}
              height={46}
            />
          </div>
        </div>
      </div>

      {dashboard.alerts.length > 0 && (
        <motion.div
          initial={{ opacity: 0, scale: 0.98 }}
          animate={{ opacity: 1, scale: 1 }}
          className="flex items-center justify-between rounded-xl border border-amber-500/30 bg-amber-500/10 px-3 py-1.5 text-[11px] text-amber-200"
        >
          <span className="inline-flex items-center gap-1.5">
            <AlertTriangle className="h-3.5 w-3.5 text-amber-400" />
            <strong>{dashboard.alerts[0].provider_name}</strong>
            <span>{t(dashboard.alerts[0].message_key)}</span>
          </span>
          <span className="font-mono font-semibold">
            {dashboard.alerts[0].utilization_pct.toFixed(0)}%
          </span>
        </motion.div>
      )}

      <div className="flex items-center justify-between px-0.5">
        <span className="inline-flex items-center gap-1.5 text-[11px] font-semibold text-white/70">
          <Layers className="h-3.5 w-3.5 text-white/45" />
          {t("overview.topUsage")}
        </span>
        <span className="font-mono text-[10px] text-white/45">
          {dashboard.providers.length} {t("overview.activeProviders")}
        </span>
      </div>

      {dashboard.providers.length === 0 ? (
        <button
          type="button"
          onClick={onOpenProviders}
          className="notch-card flex flex-col items-center justify-center gap-1.5 rounded-2xl py-10 text-center transition"
        >
          <span className="text-sm font-semibold text-white/85">
            {t("overview.noProvidersTitle")}
          </span>
          <span className="text-xs text-white/45">{t("overview.noProvidersDesc")}</span>
        </button>
      ) : (
        <div className="thin-scroll grid max-h-[246px] grid-cols-2 gap-2 overflow-y-auto pr-1">
          {dashboard.providers.map((provider, index) => (
            <motion.button
              key={provider.provider_id}
              type="button"
              onClick={() => onSelectProvider(provider.provider_id)}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{
                delay: Math.min(index * 0.035, 0.25),
                type: "spring",
                stiffness: 340,
                damping: 28,
              }}
              whileHover={{ y: -1.5 }}
              whileTap={{ scale: 0.985 }}
              className="notch-card group flex flex-col gap-2 rounded-xl p-2.5 text-left transition"
            >
              <div className="flex items-center justify-between gap-2">
                <div className="flex min-w-0 items-center gap-2">
                  <ProviderGlyph
                    kind={provider.kind}
                    shortName={provider.short_name}
                    accentHex={provider.accent_hex}
                    secondaryHex={provider.secondary_hex}
                    size="sm"
                  />
                  <div className="min-w-0">
                    <div className="truncate text-xs font-semibold text-white/95">
                      {provider.label}
                    </div>
                    <div className="truncate font-mono text-[10px] text-white/45">
                      {provider.total_tokens > 0
                        ? `${formatCompactNumber(provider.total_tokens)} tok`
                        : provider.source_note || provider.short_name}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-1">
                  <div className="text-right">
                    <div className="font-mono text-xs font-bold text-white">
                      {formatMoney(
                        provider.spend_usd,
                        dashboard.currency,
                        dashboard.usd_to_cny_rate,
                      )}
                    </div>
                    <div className="font-mono text-[10px] text-white/45">
                      {provider.utilization_pct.toFixed(0)}%
                    </div>
                  </div>
                  <ChevronRight className="h-3.5 w-3.5 text-white/25 transition group-hover:translate-x-0.5 group-hover:text-white/70" />
                </div>
              </div>

              <LiquidQuotaBar
                percent={provider.utilization_pct}
                accentHex={provider.accent_hex}
                secondaryHex={provider.secondary_hex}
                heightClass="h-1.5"
              />
            </motion.button>
          ))}
        </div>
      )}
    </motion.div>
  );
};
