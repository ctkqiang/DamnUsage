import React from "react";
import { motion } from "framer-motion";
import { ArrowUpRight, RefreshCw, Sparkles } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { DashboardReadModel } from "../../cqrs/types";
import { formatMoney } from "../../utils/format";
import { AnimatedNumber } from "../visualizations/AnimatedNumber";
import { LiquidQuotaBar } from "../visualizations/LiquidQuotaBar";
import { ProviderGlyph } from "../visualizations/ProviderGlyph";

interface PeekNotchProps {
  dashboard: DashboardReadModel;
  isSyncing: boolean;
  onExpand: () => void;
  onSyncAll: () => void;
}

export const PeekNotch: React.FC<PeekNotchProps> = ({
  dashboard,
  isSyncing,
  onExpand,
  onSyncAll,
}) => {
  const { t } = useTranslation();
  const prefix = dashboard.currency === "CNY" ? "¥" : "$";
  const totalVal =
    dashboard.currency === "CNY" ? dashboard.total_spend_cny : dashboard.total_spend_usd;
  const topThree = dashboard.providers.slice(0, 3);

  return (
    <motion.div
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -8 }}
      transition={{ duration: 0.22, ease: "easeOut" }}
      className="flex w-full flex-col gap-2.5 px-4 pb-3.5 pt-2.5"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Sparkles className="h-3.5 w-3.5 text-violet-400" />
          <span className="text-[11px] font-semibold tracking-wide text-white/85">
            {t("peek.title")}
          </span>
          <span className="rounded-full bg-white/[0.08] px-1.5 py-0.5 text-[10px] text-white/60">
            {t("peek.activeCount", { count: dashboard.providers.length })}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <AnimatedNumber
            value={totalVal}
            prefix={prefix}
            decimals={2}
            className="font-mono text-sm font-bold text-white"
          />
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onSyncAll();
            }}
            className="rounded-lg bg-white/[0.08] p-1 text-white/75 transition hover:bg-white/[0.16] hover:text-white"
          >
            <RefreshCw className={`h-3 w-3 ${isSyncing ? "animate-spin" : ""}`} />
          </button>
          <button
            type="button"
            onClick={onExpand}
            className="flex items-center gap-1 rounded-lg bg-violet-500/25 px-2 py-1 text-[10px] font-semibold text-violet-200 transition hover:bg-violet-500/40"
          >
            <span>{t("app.expand")}</span>
            <ArrowUpRight className="h-3 w-3" />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-2">
        {topThree.map((provider, idx) => (
          <motion.button
            key={provider.provider_id}
            type="button"
            onClick={onExpand}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: idx * 0.045, type: "spring", stiffness: 320, damping: 26 }}
            className="notch-card flex flex-col gap-2 rounded-xl p-2.5 text-left transition"
          >
            <div className="flex items-center gap-2">
              <ProviderGlyph
                kind={provider.kind}
                shortName={provider.short_name}
                accentHex={provider.accent_hex}
                secondaryHex={provider.secondary_hex}
                size="sm"
              />
              <div className="min-w-0 flex-1">
                <div className="truncate text-[11px] font-semibold text-white/90">
                  {provider.short_name}
                </div>
                <div className="font-mono text-[10px] text-white/55">
                  {formatMoney(provider.spend_usd, dashboard.currency, dashboard.usd_to_cny_rate)}
                </div>
              </div>
            </div>
            <LiquidQuotaBar
              percent={provider.utilization_pct}
              accentHex={provider.accent_hex}
              secondaryHex={provider.secondary_hex}
              heightClass="h-1"
            />
          </motion.button>
        ))}
      </div>
    </motion.div>
  );
};
