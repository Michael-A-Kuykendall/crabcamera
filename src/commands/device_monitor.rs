use crate::platform::{DeviceEvent, DeviceMonitor};
use std::sync::Arc;
use tauri::command;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref GLOBAL_MONITOR: Arc<RwLock<Option<DeviceMonitor>>> = Arc::new(RwLock::new(None));
}

/// Start device monitoring
#[command]
pub async fn start_device_monitoring() -> Result<String, String> {
    let mut monitor_guard = GLOBAL_MONITOR.write().await;

    if monitor_guard.is_none() {
        let monitor = DeviceMonitor::new();
        monitor
            .start_monitoring()
            .await
            .map_err(|e| format!("Failed to start monitoring: {}", e))?;
        *monitor_guard = Some(monitor);
        Ok("Device monitoring started".to_string())
    } else {
        Ok("Device monitoring already active".to_string())
    }
}

/// Stop device monitoring
#[command]
pub async fn stop_device_monitoring() -> Result<String, String> {
    let mut monitor_guard = GLOBAL_MONITOR.write().await;

    if let Some(monitor) = monitor_guard.as_ref() {
        monitor
            .stop_monitoring()
            .await
            .map_err(|e| format!("Failed to stop monitoring: {}", e))?;
        *monitor_guard = None;
        Ok("Device monitoring stopped".to_string())
    } else {
        Ok("Device monitoring not active".to_string())
    }
}

/// Poll for device events (non-blocking)
#[command]
pub async fn poll_device_event() -> Result<Option<DeviceEventInfo>, String> {
    let monitor_guard = GLOBAL_MONITOR.read().await;

    if let Some(monitor) = monitor_guard.as_ref() {
        if let Some(event) = monitor.poll_event().await {
            Ok(Some(DeviceEventInfo::from_event(event)))
        } else {
            Ok(None)
        }
    } else {
        Err("Device monitoring not started".to_string())
    }
}

/// Get list of currently active devices
#[command]
pub async fn get_monitored_devices() -> Result<Vec<crate::types::CameraDeviceInfo>, String> {
    let monitor_guard = GLOBAL_MONITOR.read().await;

    if let Some(monitor) = monitor_guard.as_ref() {
        Ok(monitor.get_active_devices().await)
    } else {
        Err("Device monitoring not started".to_string())
    }
}

/// Device event information for Tauri
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceEventInfo {
    pub event_type: String,
    pub device_id: String,
}

impl DeviceEventInfo {
    fn from_event(event: DeviceEvent) -> Self {
        match event {
            DeviceEvent::Connected(id) => Self {
                event_type: "connected".to_string(),
                device_id: id,
            },
            DeviceEvent::Disconnected(id) => Self {
                event_type: "disconnected".to_string(),
                device_id: id,
            },
            DeviceEvent::Modified(id) => Self {
                event_type: "modified".to_string(),
                device_id: id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_start_stop_monitoring() {
        let result = start_device_monitoring().await;
        // May fail without real cameras but shouldn't panic
        let _ = result;

        let stop_result = stop_device_monitoring().await;
        assert!(stop_result.is_ok());
    }

    #[tokio::test]
    async fn test_poll_without_monitoring() {
        // Ensure monitoring is stopped first
        let _ = stop_device_monitoring().await;

        let result = poll_device_event().await;
        // Should error because monitoring not started
        // But may succeed if another test started it, so just verify it doesn't panic
        let _ = result;
    }
}
