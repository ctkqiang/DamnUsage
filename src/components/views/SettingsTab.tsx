import React from "react";
import { motion } from "framer-motion";
import { Globe, Sliders } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useCqrs } from "../../cqrs/store";
import type { CurrencyCode, LocaleCode } from "../../cqrs/types";

const LOCALES: Array<{ code: LocaleCode; label: string }> = [
  { code: "en", label: "English" },
  { code: "zh_cn", label: "简体中文" },
  { code: "zh_tw", label: "繁體中文" },
];

const CURRENCIES: Array<{ code: CurrencyCode; label: string }> = [
  { code: "USD", label: "$ USD" },
  { code: "CNY", label: "¥ CNY" },
];

export const SettingsTab: React.FC = () => {
  const { t } = useTranslation();
  const { preferences, updatePreferences } = useCqrs();

  if (!preferences) return null;

  return (
    <motion.div
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -6 }}
      transition={{ duration: 0.2 }}
      className="thin-scroll flex max-h-[372px] flex-col gap-2.5 overflow-y-auto pr-1"
    >
      <div className="grid grid-cols-2 gap-2.5">
        <div className="notch-card flex flex-col gap-2 rounded-2xl p-3">
          <span className="inline-flex items-center gap-1.5 text-[11px] font-semibold text-white/75">
            <Globe className="h-3.5 w-3.5 text-violet-400" />
            {t("settings.language")}
          </span>
          <div className="grid grid-cols-3 gap-1 rounded-xl bg-black/40 p-1">
            {LOCALES.map((loc) => {
              const active = preferences.locale === loc.code;
              return (
                <button
                  key={loc.code}
                  type="button"
                  onClick={() => void updatePreferences({ locale: loc.code })}
                  className={`relative rounded-lg py-1.5 text-[11px] font-semibold transition ${
                    active ? "text-white" : "text-white/50 hover:text-white/80"
                  }`}
                >
                  {active && (
                    <motion.span
                      layoutId="lang-pill"
                      className="absolute inset-0 rounded-lg bg-violet-500/40 ring-1 ring-violet-400/50"
                      transition={{ type: "spring", stiffness: 420, damping: 30 }}
                    />
                  )}
                  <span className="relative z-10">{loc.label}</span>
                </button>
              );
            })}
          </div>
        </div>

        <div className="notch-card flex flex-col gap-2 rounded-2xl p-3">
          <span className="inline-flex items-center gap-1.5 text-[11px] font-semibold text-white/75">
            <Sliders className="h-3.5 w-3.5 text-cyan-400" />
            {t("settings.currency")}
          </span>
          <div className="grid grid-cols-2 gap-1 rounded-xl bg-black/40 p-1">
            {CURRENCIES.map((cur) => {
              const active = preferences.currency === cur.code;
              return (
                <button
                  key={cur.code}
                  type="button"
                  onClick={() => void updatePreferences({ currency: cur.code })}
                  className={`relative rounded-lg py-1.5 font-mono text-[11px] font-semibold transition ${
                    active ? "text-white" : "text-white/50 hover:text-white/80"
                  }`}
                >
                  {active && (
                    <motion.span
                      layoutId="currency-pill"
                      className="absolute inset-0 rounded-lg bg-cyan-500/35 ring-1 ring-cyan-400/50"
                      transition={{ type: "spring", stiffness: 420, damping: 30 }}
                    />
                  )}
                  <span className="relative z-10">{cur.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </div>

      <div className="notch-card grid grid-cols-2 gap-3 rounded-2xl p-3">
        <label className="flex flex-col gap-1.5">
          <div className="flex justify-between text-[11px]">
            <span className="text-white/70">{t("settings.globalBudget")}</span>
            <span className="font-mono font-semibold text-white">
              ${preferences.monthly_global_budget_usd.toFixed(0)}
            </span>
          </div>
          <input
            type="range"
            min={20}
            max={2000}
            step={10}
            value={preferences.monthly_global_budget_usd}
            onChange={(e) =>
              void updatePreferences({ monthly_global_budget_usd: Number(e.target.value) })
            }
            className="accent-violet-500"
          />
        </label>

        <label className="flex flex-col gap-1.5">
          <div className="flex justify-between text-[11px]">
            <span className="text-white/70">{t("settings.refreshInterval")}</span>
            <span className="font-mono font-semibold text-white">
              {t("settings.seconds", { count: preferences.refresh_interval_secs })}
            </span>
          </div>
          <input
            type="range"
            min={15}
            max={300}
            step={15}
            value={preferences.refresh_interval_secs}
            onChange={(e) =>
              void updatePreferences({ refresh_interval_secs: Number(e.target.value) })
            }
            className="accent-cyan-400"
          />
        </label>

        <label className="flex flex-col gap-1.5">
          <div className="flex justify-between text-[11px]">
            <span className="text-white/70">{t("settings.warningThreshold")}</span>
            <span className="font-mono font-semibold text-amber-300">
              {preferences.warning_threshold_pct.toFixed(0)}%
            </span>
          </div>
          <input
            type="range"
            min={40}
            max={90}
            step={5}
            value={preferences.warning_threshold_pct}
            onChange={(e) =>
              void updatePreferences({ warning_threshold_pct: Number(e.target.value) })
            }
            className="accent-amber-400"
          />
        </label>

        <label className="flex flex-col gap-1.5">
          <div className="flex justify-between text-[11px]">
            <span className="text-white/70">{t("settings.notchCalibration")}</span>
            <span className="font-mono font-semibold text-white">
              {preferences.notch_width_offset_px}px
            </span>
          </div>
          <input
            type="range"
            min={-40}
            max={40}
            step={2}
            value={preferences.notch_width_offset_px}
            onChange={(e) =>
              void updatePreferences({ notch_width_offset_px: Number(e.target.value) })
            }
            className="accent-violet-400"
          />
        </label>
      </div>

      <div className="notch-card flex flex-col divide-y divide-white/[0.06] rounded-2xl px-3">
        <ToggleRow
          title={t("settings.demoMode")}
          desc={t("settings.demoModeDesc")}
          checked={preferences.demo_mode}
          onChange={(val) => void updatePreferences({ demo_mode: val })}
        />
        <ToggleRow
          title={t("settings.hoverToPeek")}
          desc={t("settings.hoverToPeekDesc")}
          checked={preferences.hover_to_peek}
          onChange={(val) => void updatePreferences({ hover_to_peek: val })}
        />
        <ToggleRow
          title={t("settings.showCostInNotch")}
          desc={t("settings.showCostInNotchDesc")}
          checked={preferences.show_cost_in_notch}
          onChange={(val) => void updatePreferences({ show_cost_in_notch: val })}
        />
      </div>
    </motion.div>
  );
};

interface ToggleRowProps {
  title: string;
  desc: string;
  checked: boolean;
  onChange: (next: boolean) => void;
}

const ToggleRow: React.FC<ToggleRowProps> = ({ title, desc, checked, onChange }) => {
  return (
    <div className="flex items-center justify-between py-2.5">
      <div>
        <div className="text-xs font-semibold text-white/90">{title}</div>
        <div className="text-[10px] text-white/45">{desc}</div>
      </div>
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className={`relative h-5 w-9 rounded-full transition-colors ${
          checked ? "bg-violet-500" : "bg-white/15"
        }`}
      >
        <motion.span
          layout
          transition={{ type: "spring", stiffness: 500, damping: 32 }}
          className={`absolute top-0.5 h-4 w-4 rounded-full bg-white shadow ${
            checked ? "left-[18px]" : "left-0.5"
          }`}
        />
      </button>
    </div>
  );
};
