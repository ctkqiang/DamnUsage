import React, { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
  ChevronUp,
  LayoutGrid,
  Plug,
  RefreshCw,
  Settings,
  Sparkles,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useCqrs } from "../../cqrs/store";
import { OverviewTab } from "../views/OverviewTab";
import { ProviderDetailDrawer } from "../views/ProviderDetailDrawer";
import { ProvidersTab } from "../views/ProvidersTab";
import { SettingsTab } from "../views/SettingsTab";

type ActiveTab = "overview" | "providers" | "settings";

interface ExpandedDashboardProps {
  onCollapse: () => void;
}

export const ExpandedDashboard: React.FC<ExpandedDashboardProps> = ({ onCollapse }) => {
  const { t } = useTranslation();
  const { dashboard, preferences, isSyncing, syncAll, updatePreferences } = useCqrs();
  const [activeTab, setActiveTab] = useState<ActiveTab>("overview");
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(null);

  if (!dashboard || !preferences) return null;

  const toggleLanguageQuick = () => {
    const next = preferences.locale === "en" ? "zh_cn" : "en";
    void updatePreferences({ locale: next });
  };

  const toggleCurrencyQuick = () => {
    const next = preferences.currency === "USD" ? "CNY" : "USD";
    void updatePreferences({ currency: next });
  };

  const tabs: Array<{ id: ActiveTab; label: string; icon: React.ReactNode }> = [
    { id: "overview", label: t("tabs.overview"), icon: <LayoutGrid className="h-3.5 w-3.5" /> },
    { id: "providers", label: t("tabs.providers"), icon: <Plug className="h-3.5 w-3.5" /> },
    { id: "settings", label: t("tabs.settings"), icon: <Settings className="h-3.5 w-3.5" /> },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.22 }}
      className="relative flex w-full flex-col gap-3 px-4 pb-4 pt-3"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="flex h-6 w-6 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-cyan-400 shadow-glow">
            <Sparkles className="h-3.5 w-3.5 text-white" />
          </div>
          <span className="text-xs font-bold tracking-tight text-white">
            {t("app.name")}
          </span>
          <span
            className={`rounded-full px-2 py-0.5 font-mono text-[9px] font-bold tracking-wider ${
              dashboard.demo_mode
                ? "bg-violet-500/20 text-violet-300 ring-1 ring-violet-400/30"
                : "bg-emerald-500/20 text-emerald-300 ring-1 ring-emerald-400/30"
            }`}
          >
            {dashboard.demo_mode ? t("app.demoBadge") : t("app.liveBadge")}
          </span>
        </div>

        <div className="flex items-center gap-1 rounded-xl bg-white/[0.05] p-0.5">
          {tabs.map((tab) => {
            const active = activeTab === tab.id && !selectedProviderId;
            return (
              <button
                key={tab.id}
                type="button"
                onClick={() => {
                  setSelectedProviderId(null);
                  setActiveTab(tab.id);
                }}
                className={`relative flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[11px] font-semibold transition ${
                  active ? "text-white" : "text-white/50 hover:text-white/85"
                }`}
              >
                {active && (
                  <motion.span
                    layoutId="main-nav-pill"
                    className="absolute inset-0 rounded-lg bg-white/[0.13] shadow-sm"
                    transition={{ type: "spring", stiffness: 450, damping: 32 }}
                  />
                )}
                <span className="relative z-10 flex items-center gap-1.5">
                  {tab.icon}
                  {tab.label}
                </span>
              </button>
            );
          })}
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={toggleCurrencyQuick}
            className="rounded-lg bg-white/[0.06] px-2 py-1 font-mono text-[10px] font-bold text-white/75 transition hover:bg-white/[0.14] hover:text-white"
          >
            {preferences.currency === "CNY" ? "¥ CNY" : "$ USD"}
          </button>
          <button
            type="button"
            onClick={toggleLanguageQuick}
            className="rounded-lg bg-white/[0.06] px-2 py-1 text-[10px] font-bold text-white/75 transition hover:bg-white/[0.14] hover:text-white"
          >
            {preferences.locale === "en" ? "中文" : "EN"}
          </button>
          <button
            type="button"
            onClick={() => void syncAll()}
            className="rounded-lg bg-white/[0.06] p-1.5 text-white/75 transition hover:bg-white/[0.14] hover:text-white"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isSyncing ? "animate-spin" : ""}`} />
          </button>
          <button
            type="button"
            onClick={onCollapse}
            className="rounded-lg bg-white/[0.06] p-1.5 text-white/75 transition hover:bg-white/[0.14] hover:text-white"
          >
            <ChevronUp className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>

      <AnimatePresence mode="wait">
        {selectedProviderId ? (
          <ProviderDetailDrawer
            key={`detail-${selectedProviderId}`}
            providerId={selectedProviderId}
            onBack={() => setSelectedProviderId(null)}
          />
        ) : activeTab === "overview" ? (
          <OverviewTab
            key="overview"
            dashboard={dashboard}
            onSelectProvider={setSelectedProviderId}
            onOpenProviders={() => setActiveTab("providers")}
          />
        ) : activeTab === "providers" ? (
          <ProvidersTab key="providers" />
        ) : (
          <SettingsTab key="settings" />
        )}
      </AnimatePresence>
    </motion.div>
  );
};
