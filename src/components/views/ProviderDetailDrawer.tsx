import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { ArrowLeft, Clock, RefreshCw, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useCqrs } from "../../cqrs/store";
import type { ProviderDetailReadModel } from "../../cqrs/types";
import { formatCompactNumber, formatMoney } from "../../utils/format";
import { LiquidQuotaBar } from "../visualizations/LiquidQuotaBar";
import { ProviderGlyph } from "../visualizations/ProviderGlyph";
import { SparklineChart } from "../visualizations/SparklineChart";

interface ProviderDetailDrawerProps {
  providerId: string;
  onBack: () => void;
}

export const ProviderDetailDrawer: React.FC<ProviderDetailDrawerProps> = ({
  providerId,
  onBack,
}) => {
  const { t } = useTranslation();
  const { dashboard, loadProviderDetail, syncProvider, removeProvider, isSyncing } = useCqrs();
  const [detail, setDetail] = useState<ProviderDetailReadModel | null>(null);

  useEffect(() => {
    void loadProviderDetail(providerId).then(setDetail);
  }, [providerId, loadProviderDetail, dashboard?.last_synced_at]);

  if (!detail || !dashboard) {
    return null;
  }

  const snap = detail.snapshot;
  const currency = dashboard.currency;
  const rate = dashboard.usd_to_cny_rate;

  const handleDelete = async () => {
    await removeProvider(providerId);
    onBack();
  };

  return (
    <motion.div
      initial={{ opacity: 0, x: 18 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 18 }}
      transition={{ type: "spring", stiffness: 360, damping: 30 }}
      className="flex flex-col gap-3"
    >
      <div className="flex items-center justify-between">
        <button
          type="button"
          onClick={onBack}
          className="inline-flex items-center gap-1.5 rounded-lg bg-white/[0.06] px-2.5 py-1 text-xs font-medium text-white/80 transition hover:bg-white/[0.14] hover:text-white"
        >
          <ArrowLeft className="h-3.5 w-3.5" />
          {t("detail.back")}
        </button>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => void syncProvider(providerId)}
            className="inline-flex items-center gap-1.5 rounded-lg bg-violet-500/25 px-2.5 py-1 text-xs font-medium text-violet-100 transition hover:bg-violet-500/40"
          >
            <RefreshCw className={`h-3.5 w-3.5 ${isSyncing ? "animate-spin" : ""}`} />
            {t("detail.syncProvider")}
          </button>
          <button
            type="button"
            onClick={() => void handleDelete()}
            className="inline-flex items-center gap-1 rounded-lg bg-rose-500/15 px-2.5 py-1 text-xs font-medium text-rose-300 transition hover:bg-rose-500/30"
          >
            <Trash2 className="h-3.5 w-3.5" />
            {t("providers.remove")}
          </button>
        </div>
      </div>

      <div className="notch-card flex items-center justify-between rounded-2xl p-3.5">
        <div className="flex items-center gap-3">
          <ProviderGlyph
            kind={detail.account.kind}
            shortName={detail.catalog.short_name}
            accentHex={detail.catalog.accent_hex}
            secondaryHex={detail.catalog.secondary_hex}
            size="lg"
          />
          <div>
            <div className="text-sm font-bold text-white">{detail.account.label}</div>
            <div className="mt-0.5 flex items-center gap-2 text-[11px] text-white/50">
              <span>{snap?.source_note || detail.catalog.default_endpoint}</span>
              {snap && (
                <span className="inline-flex items-center gap-1 font-mono text-white/60">
                  <Clock className="h-3 w-3" />
                  {snap.latency_ms}ms
                </span>
              )}
            </div>
          </div>
        </div>

        <div className="text-right">
          <div className="font-mono text-xl font-bold text-white">
            {formatMoney(snap?.spend_usd ?? 0, currency, rate)}
          </div>
          {snap?.balance_usd !== null && snap?.balance_usd !== undefined && (
            <div className="font-mono text-[11px] text-emerald-400">
              {t("overview.balanceLeft")}: {formatMoney(snap.balance_usd, currency, rate)}
            </div>
          )}
        </div>
      </div>

      {snap?.error_message && (
        <div className="rounded-xl border border-rose-500/30 bg-rose-500/10 px-3 py-2 text-xs text-rose-200">
          <strong>{t("detail.errorTitle")}:</strong> {snap.error_message}
        </div>
      )}

      <div className="grid grid-cols-12 gap-2.5">
        <div className="notch-card col-span-7 flex flex-col gap-2 rounded-2xl p-3">
          <div className="flex items-center justify-between text-[11px] font-semibold text-white/75">
            <span>{t("detail.modelBreakdown")}</span>
            <span className="font-mono text-[10px] text-white/45">
              {formatCompactNumber(snap?.tokens.input_tokens ?? 0)} {t("detail.inputTokens")} ·{" "}
              {formatCompactNumber(snap?.tokens.output_tokens ?? 0)} {t("detail.outputTokens")}
            </span>
          </div>
          {snap && snap.model_breakdown.length > 0 ? (
            <div className="thin-scroll flex max-h-[168px] flex-col gap-2 overflow-y-auto pr-1">
              {snap.model_breakdown.map((m) => {
                const share =
                  snap.spend_usd > 0 ? Math.min(100, (m.spend_usd / snap.spend_usd) * 100) : 0;
                return (
                  <div key={m.model_id} className="flex flex-col gap-1">
                    <div className="flex items-center justify-between text-[11px]">
                      <span className="max-w-[200px] truncate font-mono text-white/85">
                        {m.model_id}
                      </span>
                      <span className="font-mono font-semibold text-white">
                        {formatMoney(m.spend_usd, currency, rate)}
                      </span>
                    </div>
                    <LiquidQuotaBar
                      percent={share}
                      accentHex={detail.catalog.accent_hex}
                      secondaryHex={detail.catalog.secondary_hex}
                      heightClass="h-1"
                    />
                  </div>
                );
              })}
            </div>
          ) : (
            <div className="py-6 text-center text-[11px] text-white/45">
              {t("detail.noBreakdown")}
            </div>
          )}
        </div>

        <div className="notch-card col-span-5 flex flex-col justify-between gap-2 rounded-2xl p-3">
          <span className="text-[11px] font-semibold text-white/75">
            {t("detail.quotaWindows")}
          </span>
          <div className="flex flex-col gap-2">
            {(snap?.quota_windows ?? []).map((win) => {
              const pct = win.limit > 0 ? Math.min(100, (win.used / win.limit) * 100) : 0;
              return (
                <div key={win.label} className="flex flex-col gap-1">
                  <div className="flex items-center justify-between text-[10px] text-white/65">
                    <span className="truncate">{win.label}</span>
                    <span className="font-mono text-white/85">
                      {win.used.toFixed(1)}/{win.limit.toFixed(0)} {win.unit}
                    </span>
                  </div>
                  <LiquidQuotaBar
                    percent={pct}
                    accentHex={detail.catalog.accent_hex}
                    secondaryHex={detail.catalog.secondary_hex}
                    heightClass="h-1"
                  />
                </div>
              );
            })}
          </div>
          <div className="pt-1">
            <SparklineChart
              points={detail.history}
              accentHex={detail.catalog.accent_hex}
              secondaryHex={detail.catalog.secondary_hex}
              width={215}
              height={46}
            />
          </div>
        </div>
      </div>
    </motion.div>
  );
};
