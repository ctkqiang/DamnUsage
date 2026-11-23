use damnusage_core::application::bus::CqrsApp;
use damnusage_core::application::commands::sync_usage::SyncAllProvidersCommand;
use damnusage_core::application::commands::transition_notch::TransitionNotchCommand;
use damnusage_core::application::queries::get_dashboard::GetDashboardQuery;
use damnusage_core::domain::value_objects::NotchDisplayState;
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

pub fn setup_tray(app: &AppHandle, cqrs: Arc<CqrsApp>) -> tauri::Result<()> {
    let toggle_item = MenuItem::with_id(app, "toggle_notch", "Toggle DamnUsage", true, None::<&str>)?;
    let sync_item = MenuItem::with_id(app, "sync_now", "Sync All Providers", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit_app", "Quit DamnUsage", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle_item, &sync_item, &quit_item])?;

    let mut builder = TrayIconBuilder::new().menu(&menu).tooltip("DamnUsage");
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    let menu_cqrs = cqrs.clone();
    let click_cqrs = cqrs;

    builder
        .on_menu_event(move |app_handle, event| match event.id.as_ref() {
            "toggle_notch" => {
                let runner = menu_cqrs.clone();
                tauri::async_runtime::spawn(async move {
                    let current = runner.queries.get_dashboard(GetDashboardQuery).await.notch_state;
                    let target = match current {
                        NotchDisplayState::Expanded => NotchDisplayState::Collapsed,
                        _ => NotchDisplayState::Expanded,
                    };
                    let _ = runner
                        .commands
                        .transition_notch(TransitionNotchCommand { target_state: target })
                        .await;
                });
            }
            "sync_now" => {
                let runner = menu_cqrs.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = runner
                        .commands
                        .sync_all(SyncAllProvidersCommand {
                            include_demo_if_empty: true,
                        })
                        .await;
                });
            }
            "quit_app" => {
                app_handle.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let runner = click_cqrs.clone();
                tauri::async_runtime::spawn(async move {
                    let current = runner.queries.get_dashboard(GetDashboardQuery).await.notch_state;
                    let target = match current {
                        NotchDisplayState::Expanded => NotchDisplayState::Collapsed,
                        _ => NotchDisplayState::Expanded,
                    };
                    let _ = runner
                        .commands
                        .transition_notch(TransitionNotchCommand { target_state: target })
                        .await;
                });
            }
        })
        .build(app)?;

    Ok(())
}
