import React, { createContext, useCallback, useContext, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { subscribeDomainEvents } from "./bridge";
import {
  dispatchConfigureProvider,
  dispatchRemoveProvider,
  dispatchSyncAll,
  dispatchSyncProvider,
  dispatchTransitionNotch,
  dispatchUpdatePreferences,
} from "./commands";
import {
  queryCatalog,
  queryDashboard,
  queryNotchGeometry,
  queryPreferences,
  queryProviderDetail,
} from "./queries";
import type {
  ConfigureProviderCommand,
  DashboardReadModel,
  NotchDisplayState,
  NotchGeometry,
  ProviderCatalogItemView,
  ProviderDetailReadModel,
  UpdatePreferencesCommand,
  UserPreferences,
} from "./types";

interface CqrsContextValue {
  dashboard: DashboardReadModel | null;
  catalog: ProviderCatalogItemView[];
  preferences: UserPreferences | null;
  geometry: NotchGeometry;
  isSyncing: boolean;
  refreshAllReadModels: () => Promise<void>;
  loadProviderDetail: (providerId: string) => Promise<ProviderDetailReadModel>;
  configureProvider: (cmd: ConfigureProviderCommand) => Promise<void>;
  removeProvider: (providerId: string) => Promise<void>;
  syncProvider: (providerId: string) => Promise<void>;
  syncAll: () => Promise<void>;
  updatePreferences: (cmd: UpdatePreferencesCommand) => Promise<void>;
  transitionNotch: (target: NotchDisplayState) => Promise<void>;
}

const DEFAULT_GEOMETRY: NotchGeometry = {
  has_hardware_notch: true,
  notch_height_px: 34,
  screen_width_px: 1512,
  scale_factor: 2,
};

const CqrsContext = createContext<CqrsContextValue | null>(null);

export const CqrsProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { i18n } = useTranslation();
  const [dashboard, setDashboard] = useState<DashboardReadModel | null>(null);
  const [catalog, setCatalog] = useState<ProviderCatalogItemView[]>([]);
  const [preferences, setPreferences] = useState<UserPreferences | null>(null);
  const [geometry, setGeometry] = useState<NotchGeometry>(DEFAULT_GEOMETRY);
  const [isSyncing, setIsSyncing] = useState(false);

  const refreshAllReadModels = useCallback(async () => {
    const [nextDash, nextCatalog, nextPrefs] = await Promise.all([
      queryDashboard(),
      queryCatalog(),
      queryPreferences(),
    ]);
    setDashboard(nextDash);
    setCatalog(nextCatalog);
    setPreferences(nextPrefs);
    if (i18n.language !== nextPrefs.locale) {
      await i18n.changeLanguage(nextPrefs.locale);
    }
  }, [i18n]);

  useEffect(() => {
    void refreshAllReadModels();
    void queryNotchGeometry().then(setGeometry);

    let unlisten: (() => void) | undefined;
    void subscribeDomainEvents(() => {
      void refreshAllReadModels();
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [refreshAllReadModels]);

  const loadProviderDetail = useCallback(async (providerId: string) => {
    return queryProviderDetail(providerId);
  }, []);

  const configureProvider = useCallback(
    async (cmd: ConfigureProviderCommand) => {
      setIsSyncing(true);
      try {
        await dispatchConfigureProvider(cmd);
        await refreshAllReadModels();
      } finally {
        setIsSyncing(false);
      }
    },
    [refreshAllReadModels],
  );

  const removeProvider = useCallback(
    async (providerId: string) => {
      await dispatchRemoveProvider(providerId);
      await refreshAllReadModels();
    },
    [refreshAllReadModels],
  );

  const syncProvider = useCallback(
    async (providerId: string) => {
      setIsSyncing(true);
      try {
        await dispatchSyncProvider(providerId);
        await refreshAllReadModels();
      } finally {
        setIsSyncing(false);
      }
    },
    [refreshAllReadModels],
  );

  const syncAll = useCallback(async () => {
    setIsSyncing(true);
    try {
      await dispatchSyncAll(preferences?.demo_mode ?? true);
      await refreshAllReadModels();
    } finally {
      setIsSyncing(false);
    }
  }, [preferences?.demo_mode, refreshAllReadModels]);

  const updatePreferences = useCallback(
    async (cmd: UpdatePreferencesCommand) => {
      const updated = await dispatchUpdatePreferences(cmd);
      setPreferences(updated);
      if (cmd.locale && i18n.language !== cmd.locale) {
        await i18n.changeLanguage(cmd.locale);
      }
      await refreshAllReadModels();
    },
    [i18n, refreshAllReadModels],
  );

  const transitionNotch = useCallback(
    async (target: NotchDisplayState) => {
      await dispatchTransitionNotch(target);
      setDashboard((prev) => (prev ? { ...prev, notch_state: target } : prev));
    },
    [],
  );

  return (
    <CqrsContext.Provider
      value={{
        dashboard,
        catalog,
        preferences,
        geometry,
        isSyncing,
        refreshAllReadModels,
        loadProviderDetail,
        configureProvider,
        removeProvider,
        syncProvider,
        syncAll,
        updatePreferences,
        transitionNotch,
      }}
    >
      {children}
    </CqrsContext.Provider>
  );
};

export function useCqrs(): CqrsContextValue {
  const ctx = useContext(CqrsContext);
  if (!ctx) {
    throw new Error("useCqrs must be used within CqrsProvider");
  }
  return ctx;
}
