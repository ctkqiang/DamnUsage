import React from "react";
import { motion } from "framer-motion";
import type { DashboardReadModel, UserPreferences } from "../../cqrs/types";
import { AnimatedNumber } from "../visualizations/AnimatedNumber";
import { RadialGauge } from "../visualizations/RadialGauge";

interface CollapsedNotchProps {
  dashboard: DashboardReadModel;
  preferences: UserPreferences;
  isSyncing: boolean;
  onExpand: () => void;
}

export const CollapsedNotch: React.FC<CollapsedNotchProps> = ({
  dashboard,
  preferences,
  isSyncing,
  onExpand,
}) => {
  const { headline } = dashboard;
  const currencyPrefix = dashboard.currency === "CNY" ? "¥" : "$";
  const amount =
    dashboard.currency === "CNY" ? headline.total_spend_cny : headline.total_spend_usd;

  return (
    <motion.button
      type="button"
      onClick={onExpand}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.18 }}
      className="flex h-[34px] w-full items-center justify-between px-3.5 text-left focus:outline-none"
    >
      <div className="flex items-center gap-2">
        <span className="relative flex h-2 w-2">
          <motion.span
            className="absolute inline-flex h-full w-full rounded-full opacity-75"
            style={{ backgroundColor: headline.primary_accent_hex }}
            animate={{ scale: isSyncing ? [1, 2.2, 1] : [1, 1.65, 1], opacity: [0.8, 0, 0.8] }}
            transition={{ duration: isSyncing ? 0.8 : 2.4, repeat: Infinity }}
          />
          <span
            className="relative inline-flex h-2 w-2 rounded-full"
            style={{ backgroundColor: headline.primary_accent_hex }}
          />
        </span>
        <span className="max-w-[74px] truncate text-[11px] font-semibold tracking-tight text-white/90">
          {headline.primary_provider_name}
        </span>
      </div>

      <div className="h-3.5 w-[112px] shrink-0" />

      <div className="flex items-center gap-2">
        {preferences.show_cost_in_notch && (
          <AnimatedNumber
            value={amount}
            prefix={currencyPrefix}
            decimals={2}
            className="font-mono text-[11px] font-semibold tracking-tight text-white"
          />
        )}
        <RadialGauge
          percent={headline.global_utilization_pct}
          size={18}
          strokeWidth={2.5}
          accentHex={headline.primary_accent_hex}
          secondaryHex="#00D4FF"
          showLabel={false}
        />
      </div>
    </motion.button>
  );
};
