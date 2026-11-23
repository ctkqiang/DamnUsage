import { browserFallback, invokeTauri, isTauriRuntime } from "./bridge";
import type {
  ConfigureProviderCommand,
  NotchDisplayState,
  ProviderAccount,
  UpdatePreferencesCommand,
  UsageSnapshot,
  UserPreferences,
} from "./types";

export async function dispatchConfigureProvider(
  command: ConfigureProviderCommand,
): Promise<ProviderAccount> {
  if (!isTauriRuntime()) {
    return browserFallback.configureProvider(command);
  }
  return invokeTauri<ProviderAccount>("cmd_configure_provider", { command });
}

export async function dispatchRemoveProvider(providerId: string): Promise<void> {
  if (!isTauriRuntime()) {
    browserFallback.removeProvider(providerId);
    return;
  }
  await invokeTauri<void>("cmd_remove_provider", {
    command: { provider_id: providerId },
  });
}

export async function dispatchSyncProvider(providerId: string): Promise<UsageSnapshot | null> {
  if (!isTauriRuntime()) {
    browserFallback.syncAll();
    return null;
  }
  return invokeTauri<UsageSnapshot>("cmd_sync_provider", {
    command: { provider_id: providerId },
  });
}

export async function dispatchSyncAll(includeDemoIfEmpty = true): Promise<UsageSnapshot[]> {
  if (!isTauriRuntime()) {
    return browserFallback.syncAll();
  }
  return invokeTauri<UsageSnapshot[]>("cmd_sync_all", {
    command: { include_demo_if_empty: includeDemoIfEmpty },
  });
}

export async function dispatchUpdatePreferences(
  command: UpdatePreferencesCommand,
): Promise<UserPreferences> {
  if (!isTauriRuntime()) {
    return browserFallback.updatePreferences(command);
  }
  return invokeTauri<UserPreferences>("cmd_update_preferences", { command });
}

export async function dispatchTransitionNotch(
  targetState: NotchDisplayState,
): Promise<NotchDisplayState> {
  if (!isTauriRuntime()) {
    return browserFallback.transitionNotch(targetState);
  }
  return invokeTauri<NotchDisplayState>("cmd_transition_notch", {
    command: { target_state: targetState },
  });
}
