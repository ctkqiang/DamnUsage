import React, { useState } from "react";
import { motion } from "framer-motion";
import { KeyRound, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useCqrs } from "../../cqrs/store";
import type { ProviderCatalogEntry } from "../../cqrs/types";
import { ProviderGlyph } from "../visualizations/ProviderGlyph";

interface AddProviderModalProps {
  entry: ProviderCatalogEntry;
  onClose: () => void;
}

export const AddProviderModal: React.FC<AddProviderModalProps> = ({ entry, onClose }) => {
  const { t } = useTranslation();
  const { configureProvider } = useCqrs();

  const [label, setLabel] = useState(entry.display_name);
  const [monthlyBudget, setMonthlyBudget] = useState("50");
  const [endpointOverride, setEndpointOverride] = useState("");
  const [creds, setCreds] = useState<Record<string, string>>(() => {
    const initial: Record<string, string> = {};
    for (const f of entry.credential_fields) {
      if (f.key === "aws_region") {
        initial[f.key] = "us-east-1";
      } else {
        initial[f.key] = "";
      }
    }
    return initial;
  });
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const budgetNum = Number.parseFloat(monthlyBudget);
      await configureProvider({
        provider_id: null,
        kind: entry.kind,
        label,
        enabled: true,
        monthly_budget_usd: Number.isFinite(budgetNum) && budgetNum > 0 ? budgetNum : null,
        endpoint_override: endpointOverride.trim() || null,
        credentials: creds,
        metadata: {},
      });
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="absolute inset-0 z-30 flex items-center justify-center rounded-b-[28px] bg-black/75 p-4 backdrop-blur-md"
    >
      <motion.form
        onSubmit={handleSubmit}
        initial={{ scale: 0.94, y: 12, opacity: 0 }}
        animate={{ scale: 1, y: 0, opacity: 1 }}
        exit={{ scale: 0.94, y: 12, opacity: 0 }}
        transition={{ type: "spring", stiffness: 360, damping: 28 }}
        className="thin-scroll flex max-h-[380px] w-full max-w-[460px] flex-col gap-3 overflow-y-auto rounded-2xl border border-white/15 bg-[#0d0e14] p-4 shadow-2xl"
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2.5">
            <ProviderGlyph
              kind={entry.kind}
              shortName={entry.short_name}
              accentHex={entry.accent_hex}
              secondaryHex={entry.secondary_hex}
              size="md"
            />
            <div>
              <h3 className="text-sm font-bold text-white">
                {t("providers.configureTitle", { name: entry.display_name })}
              </h3>
              <span className="text-[10px] text-white/45">
                {t(`providers.authType.${entry.auth_strategy}`)}
              </span>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg p-1 text-white/50 transition hover:bg-white/10 hover:text-white"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="grid grid-cols-2 gap-2.5">
          <label className="flex flex-col gap-1">
            <span className="text-[10px] font-medium text-white/60">
              {t("providers.labelField")}
            </span>
            <input
              type="text"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              className="rounded-lg border border-white/10 bg-white/[0.05] px-2.5 py-1.5 text-xs text-white outline-none focus:border-violet-400"
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="text-[10px] font-medium text-white/60">
              {t("providers.monthlyBudget")}
            </span>
            <input
              type="number"
              min="1"
              step="1"
              value={monthlyBudget}
              onChange={(e) => setMonthlyBudget(e.target.value)}
              className="rounded-lg border border-white/10 bg-white/[0.05] px-2.5 py-1.5 font-mono text-xs text-white outline-none focus:border-violet-400"
            />
          </label>
        </div>

        <div className="flex flex-col gap-2">
          {entry.credential_fields.map((field) => (
            <label key={field.key} className="flex flex-col gap-1">
              <span className="inline-flex items-center gap-1 text-[10px] font-medium text-white/65">
                <KeyRound className="h-3 w-3 text-violet-400" />
                {t(field.label_key)}
                {field.required && <span className="text-rose-400">*</span>}
              </span>
              <input
                type={field.secret ? "password" : "text"}
                required={field.required}
                placeholder={field.placeholder}
                value={creds[field.key] ?? ""}
                onChange={(e) =>
                  setCreds((prev) => ({ ...prev, [field.key]: e.target.value }))
                }
                className="rounded-lg border border-white/10 bg-white/[0.05] px-2.5 py-1.5 font-mono text-xs text-white placeholder-white/25 outline-none focus:border-violet-400"
              />
            </label>
          ))}

          <label className="flex flex-col gap-1">
            <span className="text-[10px] font-medium text-white/55">
              {t("providers.endpointOverride")}
            </span>
            <input
              type="text"
              placeholder={entry.default_endpoint}
              value={endpointOverride}
              onChange={(e) => setEndpointOverride(e.target.value)}
              className="rounded-lg border border-white/10 bg-white/[0.04] px-2.5 py-1.5 font-mono text-xs text-white placeholder-white/25 outline-none focus:border-violet-400"
            />
          </label>
        </div>

        {error && (
          <div className="rounded-lg bg-rose-500/15 px-2.5 py-1.5 text-[11px] text-rose-300">
            {error}
          </div>
        )}

        <div className="mt-1 flex items-center justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg bg-white/[0.07] px-3 py-1.5 text-xs font-medium text-white/75 transition hover:bg-white/[0.14]"
          >
            {t("providers.cancel")}
          </button>
          <button
            type="submit"
            disabled={submitting}
            className="rounded-lg px-3.5 py-1.5 text-xs font-semibold text-white shadow-lg transition disabled:opacity-50"
            style={{
              background: `linear-gradient(135deg, ${entry.accent_hex} 0%, ${entry.secondary_hex} 100%)`,
            }}
          >
            {t("providers.saveAndSync")}
          </button>
        </div>
      </motion.form>
    </motion.div>
  );
};
