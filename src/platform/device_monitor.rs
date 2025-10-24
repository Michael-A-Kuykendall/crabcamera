//! Device monitoring and hot-plug detection
//!
//! Provides cross-platform device monitoring to detect camera connect/disconnect events
//! and enable automatic reconnection.

use crate::types::{CameraDeviceInfo, Platform};
use crate::errors::CameraError;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;

/// Device event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    Connected(String),    // Device ID
    Disconnected(String), // Device ID
    Modified(String),     // Device ID (settings changed)
}

/// Device monitor for detecting camera changes
pub struct DeviceMonitor {
    platform: Platform,
    active_devices: Arc<RwLock<HashMap<String, CameraDeviceInfo>>>,
    event_sender: mpsc::UnboundedSender<DeviceEvent>,
    event_receiver: Arc<RwLock<mpsc::UnboundedReceiver<DeviceEvent>>>,
    is_monitoring: Arc<RwLock<bool>>,
}

impl DeviceMonitor {
    /// Create a new device monitor
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        
        Self {
            platform: Platform::current(),
            active_devices: Arc::new(RwLock::new(HashMap::new())),
            event_sender: tx,
            event_receiver: Arc::new(RwLock::new(rx)),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start monitoring for device changes
    pub async fn start_monitoring(&self) -> Result<(), CameraError> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if *is_monitoring {
            return Ok(());
        }
        
        log::info!("Starting device monitoring for platform: {:?}", self.platform);
        
        match self.platform {
            Platform::Windows => self.start_windows_monitoring().await?,
            Platform::MacOS => self.start_macos_monitoring().await?,
            Platform::Linux => self.start_linux_monitoring().await?,
            Platform::Unknown => {
                log::warn!("Device monitoring not supported on unknown platform");
                return Err(CameraError::InitializationError(
                    "Device monitoring not supported on this platform".to_string()
                ));
            }
        }
        
        *is_monitoring = true;
        Ok(())
    }
    
    /// Stop monitoring for device changes
    pub async fn stop_monitoring(&self) -> Result<(), CameraError> {
        let mut is_monitoring = self.is_monitoring.write().await;
        if !*is_monitoring {
            return Ok(());
        }
        
        log::info!("Stopping device monitoring");
        *is_monitoring = false;
        Ok(())
    }
    
    /// Get next device event (non-blocking)
    pub async fn poll_event(&self) -> Option<DeviceEvent> {
        let mut rx = self.event_receiver.write().await;
        rx.try_recv().ok()
    }
    
    /// Wait for next device event (blocking)
    pub async fn wait_for_event(&self) -> Option<DeviceEvent> {
        let mut rx = self.event_receiver.write().await;
        rx.recv().await
    }
    
    /// Get list of currently active devices
    pub async fn get_active_devices(&self) -> Vec<CameraDeviceInfo> {
        let devices = self.active_devices.read().await;
        devices.values().cloned().collect()
    }
    
    /// Update active device list
    async fn update_active_devices(&self, new_devices: Vec<CameraDeviceInfo>) {
        let mut active = self.active_devices.write().await;
        let old_ids: Vec<String> = active.keys().cloned().collect();
        let new_ids: Vec<String> = new_devices.iter().map(|d| d.id.clone()).collect();
        
        // Detect disconnections
        for old_id in &old_ids {
            if !new_ids.contains(old_id) {
                log::info!("Device disconnected: {}", old_id);
                let _ = self.event_sender.send(DeviceEvent::Disconnected(old_id.clone()));
            }
        }
        
        // Detect connections
        for device in new_devices {
            if !old_ids.contains(&device.id) {
                log::info!("Device connected: {}", device.id);
                let _ = self.event_sender.send(DeviceEvent::Connected(device.id.clone()));
            }
            active.insert(device.id.clone(), device);
        }
        
        // Remove disconnected devices
        active.retain(|id, _| new_ids.contains(id));
    }
    
    /// Windows-specific device monitoring
    #[cfg(target_os = "windows")]
    async fn start_windows_monitoring(&self) -> Result<(), CameraError> {
        use std::time::Duration;
        
        log::info!("Starting Windows device monitoring via polling");
        
        // Initial device scan
        let initial_devices = self.scan_devices_sync()?;
        self.update_active_devices(initial_devices).await;
        
        // Spawn polling task
        let active_devices = self.active_devices.clone();
        let event_sender = self.event_sender.clone();
        let is_monitoring = self.is_monitoring.clone();
        
        tokio::spawn(async move {
            while *is_monitoring.read().await {
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                if let Ok(devices) = DeviceMonitor::scan_devices_windows() {
                    let mut active = active_devices.write().await;
                    let old_ids: Vec<String> = active.keys().cloned().collect();
                    let new_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
                    
                    // Check for changes
                    for old_id in &old_ids {
                        if !new_ids.contains(old_id) {
                            log::info!("Device disconnected: {}", old_id);
                            let _ = event_sender.send(DeviceEvent::Disconnected(old_id.clone()));
                        }
                    }
                    
                    for device in devices {
                        if !old_ids.contains(&device.id) {
                            log::info!("Device connected: {}", device.id);
                            let _ = event_sender.send(DeviceEvent::Connected(device.id.clone()));
                        }
                        active.insert(device.id.clone(), device);
                    }
                    
                    active.retain(|id, _| new_ids.contains(id));
                }
            }
        });
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    async fn start_windows_monitoring(&self) -> Result<(), CameraError> {
        Err(CameraError::InitializationError("Not on Windows".to_string()))
    }
    
    /// macOS-specific device monitoring
    #[cfg(target_os = "macos")]
    async fn start_macos_monitoring(&self) -> Result<(), CameraError> {
        use std::time::Duration;
        
        log::info!("Starting macOS device monitoring via polling");
        
        // Initial device scan
        let initial_devices = self.scan_devices_sync()?;
        self.update_active_devices(initial_devices).await;
        
        // Spawn polling task
        let active_devices = self.active_devices.clone();
        let event_sender = self.event_sender.clone();
        let is_monitoring = self.is_monitoring.clone();
        
        tokio::spawn(async move {
            while *is_monitoring.read().await {
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                if let Ok(devices) = DeviceMonitor::scan_devices_macos() {
                    let mut active = active_devices.write().await;
                    let old_ids: Vec<String> = active.keys().cloned().collect();
                    let new_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
                    
                    for old_id in &old_ids {
                        if !new_ids.contains(old_id) {
                            log::info!("Device disconnected: {}", old_id);
                            let _ = event_sender.send(DeviceEvent::Disconnected(old_id.clone()));
                        }
                    }
                    
                    for device in devices {
                        if !old_ids.contains(&device.id) {
                            log::info!("Device connected: {}", device.id);
                            let _ = event_sender.send(DeviceEvent::Connected(device.id.clone()));
                        }
                        active.insert(device.id.clone(), device);
                    }
                    
                    active.retain(|id, _| new_ids.contains(id));
                }
            }
        });
        
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    async fn start_macos_monitoring(&self) -> Result<(), CameraError> {
        Err(CameraError::InitializationError("Not on macOS".to_string()))
    }
    
    /// Linux-specific device monitoring
    #[cfg(target_os = "linux")]
    async fn start_linux_monitoring(&self) -> Result<(), CameraError> {
        use std::time::Duration;
        
        log::info!("Starting Linux device monitoring via polling");
        
        // Initial device scan
        let initial_devices = self.scan_devices_sync()?;
        self.update_active_devices(initial_devices).await;
        
        // Spawn polling task
        let active_devices = self.active_devices.clone();
        let event_sender = self.event_sender.clone();
        let is_monitoring = self.is_monitoring.clone();
        
        tokio::spawn(async move {
            while *is_monitoring.read().await {
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                if let Ok(devices) = DeviceMonitor::scan_devices_linux() {
                    let mut active = active_devices.write().await;
                    let old_ids: Vec<String> = active.keys().cloned().collect();
                    let new_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();
                    
                    for old_id in &old_ids {
                        if !new_ids.contains(old_id) {
                            log::info!("Device disconnected: {}", old_id);
                            let _ = event_sender.send(DeviceEvent::Disconnected(old_id.clone()));
                        }
                    }
                    
                    for device in devices {
                        if !old_ids.contains(&device.id) {
                            log::info!("Device connected: {}", device.id);
                            let _ = event_sender.send(DeviceEvent::Connected(device.id.clone()));
                        }
                        active.insert(device.id.clone(), device);
                    }
                    
                    active.retain(|id, _| new_ids.contains(id));
                }
            }
        });
        
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    async fn start_linux_monitoring(&self) -> Result<(), CameraError> {
        Err(CameraError::InitializationError("Not on Linux".to_string()))
    }
    
    /// Synchronous device scan helper
    fn scan_devices_sync(&self) -> Result<Vec<CameraDeviceInfo>, CameraError> {
        match self.platform {
            Platform::Windows => Self::scan_devices_windows(),
            Platform::MacOS => Self::scan_devices_macos(),
            Platform::Linux => Self::scan_devices_linux(),
            Platform::Unknown => Ok(Vec::new()),
        }
    }
    
    /// Scan Windows devices
    #[cfg(target_os = "windows")]
    fn scan_devices_windows() -> Result<Vec<CameraDeviceInfo>, CameraError> {

        use nokhwa::query;
        
        let cameras = query(nokhwa::utils::ApiBackend::Auto)
            .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
        
        Ok(cameras.into_iter().map(|info| {
            CameraDeviceInfo::new(
                format!("{}", info.index().as_index().unwrap_or(0)),
                info.human_name().to_string()
            )
        }).collect())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn scan_devices_windows() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        Ok(Vec::new())
    }
    
    /// Scan macOS devices
    #[cfg(target_os = "macos")]
    fn scan_devices_macos() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        use nokhwa::query;
        
        let cameras = query(nokhwa::utils::ApiBackend::Auto)
            .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
        
        Ok(cameras.into_iter().map(|info| {
            CameraDeviceInfo::new(
                format!("{}", info.index().as_index().unwrap_or(0)),
                info.human_name().to_string()
            )
        }).collect())
    }
    
    #[cfg(not(target_os = "macos"))]
    fn scan_devices_macos() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        Ok(Vec::new())
    }
    
    /// Scan Linux devices
    #[cfg(target_os = "linux")]
    fn scan_devices_linux() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        use nokhwa::query;
        
        let cameras = query(nokhwa::utils::ApiBackend::Auto)
            .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;
        
        Ok(cameras.into_iter().map(|info| {
            CameraDeviceInfo::new(
                format!("{}", info.index().as_index().unwrap_or(0)),
                info.human_name().to_string()
            )
        }).collect())
    }
    
    #[cfg(not(target_os = "linux"))]
    fn scan_devices_linux() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        Ok(Vec::new())
    }
}

impl Default for DeviceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_device_monitor_creation() {
        let monitor = DeviceMonitor::new();
        assert!(!*monitor.is_monitoring.read().await);
    }
    
    #[tokio::test]
    async fn test_start_stop_monitoring() {
        let monitor = DeviceMonitor::new();
        
        // Start monitoring
        let result = monitor.start_monitoring().await;
        // May fail on CI without cameras, but should not panic
        let _ = result;
        
        // Stop monitoring
        let stop_result = monitor.stop_monitoring().await;
        assert!(stop_result.is_ok());
    }
    
    #[test]
    fn test_device_event_types() {
        let event1 = DeviceEvent::Connected("test".to_string());
        let event2 = DeviceEvent::Disconnected("test".to_string());
        let event3 = DeviceEvent::Modified("test".to_string());
        
        assert_ne!(event1, event2);
        assert_ne!(event2, event3);
    }
}
