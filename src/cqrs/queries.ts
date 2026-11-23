import { browserFallback, fetchNotchGeometry, invokeTauri, isTauriRuntime } from "./bridge";
import type {
  DashboardReadModel,
  NotchGeometry,
  ProviderCatalogItemView,
  ProviderDetailReadModel,
  UserPreferences,
} from "./types";

export async function queryDashboard(): Promise<DashboardReadModel> {
  if (!isTauriRuntime()) {
    return browserFallback.getDashboard();
  }
  return invokeTauri<DashboardReadModel>("qry_get_dashboard");
}

export async function queryProviderDetail(
  providerId: string,
): Promise<ProviderDetailReadModel> {
  if (!isTauriRuntime()) {
    return browserFallback.getProviderDetail(providerId);
  }
  return invokeTauri<ProviderDetailReadModel>("qry_get_provider_detail", {
    query: { provider_id: providerId },
  });
}

export async function queryCatalog(): Promise<ProviderCatalogItemView[]> {
  if (!isTauriRuntime()) {
    return browserFallback.getCatalog();
  }
  return invokeTauri<ProviderCatalogItemView[]>("qry_list_catalog");
}

export async function queryPreferences(): Promise<UserPreferences> {
  if (!isTauriRuntime()) {
    return browserFallback.getPreferences();
  }
  return invokeTauri<UserPreferences>("qry_get_preferences");
}

export async function queryNotchGeometry(): Promise<NotchGeometry> {
  return fetchNotchGeometry();
}
