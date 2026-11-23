pub mod interface;

use damnusage_core::application::bus::CqrsApp;
use damnusage_core::application::commands::sync_usage::SyncAllProvidersCommand;
use damnusage_core::application::queries::get_preferences::GetPreferencesQuery;
use damnusage_core::domain::value_objects::NotchDisplayState;
use damnusage_core::infrastructure::persistence::json_event_store::JsonEventStore;
use damnusage_core::infrastructure::persistence::secret_vault::FileSecretVault;
use damnusage_core::infrastructure::providers::demo_simulator::DemoUsageSimulator;
use damnusage_core::infrastructure::providers::registry::build_provider_registry;
use interface::ipc;
use interface::notch_window::{TauriEventPublisher, TauriNotchWindow};
use interface::tray;
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;

pub struct AppContainer {
    pub cqrs: Arc<CqrsApp>,
    pub notch_window: Arc<TauriNotchWindow>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("damnusage"));

            let repository = Arc::new(
                JsonEventStore::open(data_dir.clone()).expect("failed to initialize event store"),
            );
            let vault = Arc::new(
                FileSecretVault::open(data_dir).expect("failed to initialize secret vault"),
            );
            let notch_window = Arc::new(TauriNotchWindow::new(app.handle().clone()));
            let publisher = Arc::new(TauriEventPublisher::new(app.handle().clone()));

            let cqrs = Arc::new(CqrsApp::new(
                repository,
                vault,
                build_provider_registry(),
                Arc::new(DemoUsageSimulator::new()),
                notch_window.clone(),
                publisher,
            ));

            notch_window.configure_native_surface();
            let _ = notch_window.position_window_for_state(NotchDisplayState::Collapsed);

            let boot_cqrs = cqrs.clone();
            tauri::async_runtime::spawn(async move {
                let _ = boot_cqrs.bootstrap().await;
            });

            let poller_cqrs = cqrs.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let prefs = poller_cqrs.queries.get_preferences(GetPreferencesQuery).await;
                    tokio::time::sleep(Duration::from_secs(prefs.refresh_interval_secs.max(15))).await;
                    let _ = poller_cqrs
                        .commands
                        .sync_all(SyncAllProvidersCommand {
                            include_demo_if_empty: prefs.demo_mode,
                        })
                        .await;
                }
            });

            tray::setup_tray(app.handle(), cqrs.clone())?;

            app.manage(AppContainer { cqrs, notch_window });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::cmd_configure_provider,
            ipc::cmd_remove_provider,
            ipc::cmd_sync_provider,
            ipc::cmd_sync_all,
            ipc::cmd_update_preferences,
            ipc::cmd_transition_notch,
            ipc::qry_get_dashboard,
            ipc::qry_get_provider_detail,
            ipc::qry_list_catalog,
            ipc::qry_get_preferences,
            ipc::qry_notch_geometry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DamnUsage");
}
