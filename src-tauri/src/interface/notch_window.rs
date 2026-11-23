use async_trait::async_trait;
use damnusage_core::domain::errors::{DomainError, DomainResult};
use damnusage_core::domain::events::DomainEventEnvelope;
use damnusage_core::domain::ports::{EventPublisherPort, NotchWindowPort};
use damnusage_core::domain::value_objects::NotchDisplayState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Position, Size};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotchGeometry {
    pub has_hardware_notch: bool,
    pub notch_height_px: f64,
    pub screen_width_px: f64,
    pub scale_factor: f64,
}

pub struct TauriEventPublisher {
    app: AppHandle,
}

impl TauriEventPublisher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait]
impl EventPublisherPort for TauriEventPublisher {
    async fn publish(&self, event: DomainEventEnvelope) {
        let _ = self.app.emit("damnusage://domain-event", event);
    }
}

pub struct TauriNotchWindow {
    app: AppHandle,
}

impl TauriNotchWindow {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn detect_geometry(&self) -> NotchGeometry {
        self.app
            .get_webview_window("notch")
            .and_then(|w| {
                w.current_monitor()
                    .ok()
                    .flatten()
                    .or_else(|| w.primary_monitor().ok().flatten())
            })
            .map(|m| {
                let scale = m.scale_factor();
                let logical_w = m.size().width as f64 / scale;
                let work_top = (m.work_area().position.y - m.position().y).max(0) as f64 / scale;
                let has_hardware_notch = work_top >= 30.0;
                NotchGeometry {
                    has_hardware_notch,
                    notch_height_px: if work_top > 0.0 { work_top } else { 32.0 },
                    screen_width_px: logical_w,
                    scale_factor: scale,
                }
            })
            .unwrap_or(NotchGeometry {
                has_hardware_notch: true,
                notch_height_px: 34.0,
                screen_width_px: 1512.0,
                scale_factor: 2.0,
            })
    }

    pub fn configure_native_surface(&self) {
        #[cfg(target_os = "macos")]
        {
            if let Some(window) = self.app.get_webview_window("notch") {
                if let Ok(ns_window_ptr) = window.ns_window() {
                    apply_macos_notch_panel(ns_window_ptr as *mut objc::runtime::Object);
                }
            }
        }
    }

    pub fn position_window_for_state(&self, state: NotchDisplayState) -> DomainResult<()> {
        let Some(window) = self.app.get_webview_window("notch") else {
            return Err(DomainError::StorageError("notch window not found".to_string()));
        };

        let (width, height) = match state {
            NotchDisplayState::Collapsed => (380.0_f64, 46.0_f64),
            NotchDisplayState::Peek => (480.0_f64, 210.0_f64),
            NotchDisplayState::Expanded => (700.0_f64, 510.0_f64),
        };

        let geom = self.detect_geometry();
        let x = ((geom.screen_width_px - width) / 2.0).max(0.0);

        let _ = window.set_size(Size::Logical(LogicalSize::new(width, height)));
        let _ = window.set_position(Position::Logical(LogicalPosition::new(x, 0.0)));

        if state == NotchDisplayState::Expanded {
            let _ = window.set_focus();
        }
        Ok(())
    }
}

#[async_trait]
impl NotchWindowPort for TauriNotchWindow {
    async fn apply_state(&self, state: NotchDisplayState) -> DomainResult<()> {
        self.position_window_for_state(state)
    }
}

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
fn apply_macos_notch_panel(ns_window: *mut objc::runtime::Object) {
    use objc::runtime::NO;
    use objc::{msg_send, sel, sel_impl};

    if ns_window.is_null() {
        return;
    }

    unsafe {
        let level: i64 = 25;
        let _: () = msg_send![ns_window, setLevel: level];
        let behavior: u64 = (1 << 0) | (1 << 4) | (1 << 8);
        let _: () = msg_send![ns_window, setCollectionBehavior: behavior];
        let _: () = msg_send![ns_window, setHasShadow: NO];
        let _: () = msg_send![ns_window, setOpaque: NO];
        let _: () = msg_send![ns_window, setMovableByWindowBackground: NO];
    }
}
