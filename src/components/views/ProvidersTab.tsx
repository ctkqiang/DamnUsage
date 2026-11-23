import React, { useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { CheckCircle2, Plus, Search } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useCqrs } from "../../cqrs/store";
import type { ProviderCatalogEntry, ProviderCategory } from "../../cqrs/types";
import { ProviderGlyph } from "../visualizations/ProviderGlyph";
import { AddProviderModal } from "./AddProviderModal";

type FilterCategory = "all" | ProviderCategory;

const CATEGORIES: FilterCategory[] = [
  "all",
  "coding_agent",
  "cloud_platform",
  "aggregator",
  "global_api",
  "chinese_api",
];

export const ProvidersTab: React.FC = () => {
  const { t } = useTranslation();
  const { catalog } = useCqrs();
  const [search, setSearch] = useState("");
  const [category, setCategory] = useState<FilterCategory>("all");
  const [activeEntry, setActiveEntry] = useState<ProviderCatalogEntry | null>(null);

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase();
    return catalog.filter((item) => {
      if (category !== "all" && item.entry.category !== category) {
        return false;
      }
      if (!q) return true;
      return (
        item.entry.display_name.toLowerCase().includes(q) ||
        item.entry.short_name.toLowerCase().includes(q) ||
        item.entry.kind.toLowerCase().includes(q)
      );
    });
  }, [catalog, search, category]);

  return (
    <motion.div
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -6 }}
      transition={{ duration: 0.2 }}
      className="flex flex-col gap-2.5"
    >
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
          <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-white/35" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t("providers.searchPlaceholder")}
            className="w-full rounded-xl border border-white/10 bg-white/[0.05] py-1.5 pl-8 pr-3 text-xs text-white placeholder-white/35 outline-none focus:border-violet-400"
          />
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-1.5">
        {CATEGORIES.map((cat) => {
          const selected = category === cat;
          return (
            <button
              key={cat}
              type="button"
              onClick={() => setCategory(cat)}
              className={`relative rounded-lg px-2.5 py-1 text-[10px] font-semibold transition ${
                selected ? "text-white" : "bg-white/[0.04] text-white/55 hover:text-white/85"
              }`}
            >
              {selected && (
                <motion.span
                  layoutId="provider-cat-pill"
                  className="absolute inset-0 rounded-lg bg-violet-500/35 ring-1 ring-violet-400/50"
                  transition={{ type: "spring", stiffness: 420, damping: 32 }}
                />
              )}
              <span className="relative z-10">{t(`providers.categories.${cat}`)}</span>
            </button>
          );
        })}
      </div>

      <div className="thin-scroll grid max-h-[290px] grid-cols-2 gap-2 overflow-y-auto pr-1">
        {filtered.map((item, idx) => (
          <motion.button
            key={item.entry.kind}
            type="button"
            onClick={() => setActiveEntry(item.entry)}
            initial={{ opacity: 0, scale: 0.97 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ delay: Math.min(idx * 0.02, 0.2) }}
            whileHover={{ y: -1 }}
            whileTap={{ scale: 0.985 }}
            className="notch-card flex items-center justify-between gap-2 rounded-xl p-2.5 text-left transition"
          >
            <div className="flex min-w-0 items-center gap-2.5">
              <ProviderGlyph
                kind={item.entry.kind}
                shortName={item.entry.short_name}
                accentHex={item.entry.accent_hex}
                secondaryHex={item.entry.secondary_hex}
                size="sm"
              />
              <div className="min-w-0">
                <div className="truncate text-xs font-semibold text-white/95">
                  {item.entry.display_name}
                </div>
                <div className="truncate text-[10px] text-white/45">
                  {t(`providers.authType.${item.entry.auth_strategy}`)}
                </div>
              </div>
            </div>

            {item.configured_count > 0 ? (
              <span className="inline-flex shrink-0 items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-semibold text-emerald-300">
                <CheckCircle2 className="h-3 w-3" />
                {item.configured_count}
              </span>
            ) : (
              <span className="inline-flex shrink-0 items-center gap-0.5 rounded-lg bg-white/[0.08] px-2 py-1 text-[10px] font-semibold text-white/75">
                <Plus className="h-3 w-3" />
                {t("providers.addProvider")}
              </span>
            )}
          </motion.button>
        ))}
      </div>

      <AnimatePresence>
        {activeEntry && (
          <AddProviderModal entry={activeEntry} onClose={() => setActiveEntry(null)} />
        )}
      </AnimatePresence>
    </motion.div>
  );
};
