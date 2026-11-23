use crate::interface::notch_window::NotchGeometry;
use crate::AppContainer;
use damnusage_core::application::commands::configure_provider::ConfigureProviderCommand;
use damnusage_core::application::commands::remove_provider::RemoveProviderCommand;
use damnusage_core::application::commands::sync_usage::{
    SyncAllProvidersCommand, SyncProviderUsageCommand,
};
use damnusage_core::application::commands::transition_notch::TransitionNotchCommand;
use damnusage_core::application::commands::update_preferences::UpdatePreferencesCommand;
use damnusage_core::application::queries::get_dashboard::{DashboardReadModel, GetDashboardQuery};
use damnusage_core::application::queries::get_preferences::GetPreferencesQuery;
use damnusage_core::application::queries::get_provider_detail::{
    GetProviderDetailQuery, ProviderDetailReadModel,
};
use damnusage_core::application::queries::list_providers::{
    ListProviderCatalogQuery, ProviderCatalogItemView,
};
use damnusage_core::domain::entities::{ProviderAccount, UsageSnapshot, UserPreferences};
use damnusage_core::domain::value_objects::NotchDisplayState;
use tauri::State;

#[tauri::command]
pub async fn cmd_configure_provider(
    state: State<'_, AppContainer>,
    command: ConfigureProviderCommand,
) -> Result<ProviderAccount, String> {
    let account = state
        .cqrs
        .commands
        .configure_provider(command)
        .await
        .map_err(|e| e.to_string())?;
    let _ = state
        .cqrs
        .commands
        .sync_provider(SyncProviderUsageCommand {
            provider_id: account.id.clone(),
        })
        .await;
    Ok(account)
}

#[tauri::command]
pub async fn cmd_remove_provider(
    state: State<'_, AppContainer>,
    command: RemoveProviderCommand,
) -> Result<(), String> {
    state
        .cqrs
        .commands
        .remove_provider(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_sync_provider(
    state: State<'_, AppContainer>,
    command: SyncProviderUsageCommand,
) -> Result<UsageSnapshot, String> {
    state
        .cqrs
        .commands
        .sync_provider(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_sync_all(
    state: State<'_, AppContainer>,
    command: SyncAllProvidersCommand,
) -> Result<Vec<UsageSnapshot>, String> {
    state
        .cqrs
        .commands
        .sync_all(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_update_preferences(
    state: State<'_, AppContainer>,
    command: UpdatePreferencesCommand,
) -> Result<UserPreferences, String> {
    state
        .cqrs
        .commands
        .update_preferences(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cmd_transition_notch(
    state: State<'_, AppContainer>,
    command: TransitionNotchCommand,
) -> Result<NotchDisplayState, String> {
    state
        .cqrs
        .commands
        .transition_notch(command)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn qry_get_dashboard(state: State<'_, AppContainer>) -> Result<DashboardReadModel, String> {
    Ok(state.cqrs.queries.get_dashboard(GetDashboardQuery).await)
}

#[tauri::command]
pub async fn qry_get_provider_detail(
    state: State<'_, AppContainer>,
    query: GetProviderDetailQuery,
) -> Result<ProviderDetailReadModel, String> {
    state
        .cqrs
        .queries
        .get_provider_detail(query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn qry_list_catalog(
    state: State<'_, AppContainer>,
) -> Result<Vec<ProviderCatalogItemView>, String> {
    Ok(state
        .cqrs
        .queries
        .list_catalog(ListProviderCatalogQuery)
        .await)
}

#[tauri::command]
pub async fn qry_get_preferences(state: State<'_, AppContainer>) -> Result<UserPreferences, String> {
    Ok(state
        .cqrs
        .queries
        .get_preferences(GetPreferencesQuery)
        .await)
}

#[tauri::command]
pub async fn qry_notch_geometry(state: State<'_, AppContainer>) -> Result<NotchGeometry, String> {
    Ok(state.notch_window.detect_geometry())
}
