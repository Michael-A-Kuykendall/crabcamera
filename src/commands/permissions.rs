use crate::permissions::{check_permission_detailed, PermissionInfo, PermissionStatus};
use tauri::command;

/// Request camera permission (platform-specific)
#[command]
pub async fn request_camera_permission() -> Result<PermissionInfo, String> {
    log::info!("Requesting camera permission");

    let current_status = check_permission_detailed();

    if current_status.status == PermissionStatus::Granted {
        log::info!("Permission already granted");
        return Ok(current_status);
    }

    if !current_status.can_request {
        log::warn!("Cannot request permission: {}", current_status.message);
        return Ok(current_status);
    }

    // Platform-specific permission request
    #[cfg(target_os = "macos")]
    {
        request_permission_macos().await
    }

    #[cfg(target_os = "windows")]
    {
        // Windows doesn't have programmatic permission request
        // User must enable in Settings > Privacy > Camera
        Ok(PermissionInfo {
            status: PermissionStatus::NotDetermined,
            message: "Please enable camera access in Windows Settings > Privacy > Camera"
                .to_string(),
            can_request: false,
        })
    }

    #[cfg(target_os = "linux")]
    {
        // Linux permissions are group-based
        // User must add themselves to video group
        Ok(PermissionInfo {
            status: PermissionStatus::NotDetermined,
            message: "Run: sudo usermod -a -G video $USER && newgrp video".to_string(),
            can_request: false,
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Platform not supported".to_string())
    }
}

#[cfg(target_os = "macos")]
async fn request_permission_macos() -> Result<PermissionInfo, String> {
    use block::ConcreteBlock;
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;
    use std::sync::mpsc;
    use std::time::Duration;

    log::info!("Requesting macOS camera permission");

    unsafe {
        let av_capture_device_class =
            Class::get("AVCaptureDevice").ok_or("AVFoundation not available")?;

        let av_media_type_video = CString::new("vide").unwrap();
        let media_type: *mut Object =
            msg_send![av_capture_device_class, mediaTypeForString: av_media_type_video.as_ptr()];

        let (tx, rx) = mpsc::channel();

        // Create a proper Objective-C block using the block crate
        // This replaces the invalid inline ^(granted: bool) {} syntax
        let tx_clone = tx.clone();
        let handler = ConcreteBlock::new(move |granted: bool| {
            let _ = tx_clone.send(granted);
        });
        // Copy the block to the heap so it survives the async callback
        let handler = handler.copy();

        // Request access (this will show system dialog)
        let _: () = msg_send![av_capture_device_class, requestAccessForMediaType:media_type completionHandler:&*handler]; // Wait for user response (with timeout)
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(granted) if granted => {
                log::info!("Camera permission granted");
                Ok(PermissionInfo {
                    status: PermissionStatus::Granted,
                    message: "Camera access authorized".to_string(),
                    can_request: false,
                })
            }
            Ok(_) => {
                log::warn!("Camera permission denied");
                Ok(PermissionInfo {
                    status: PermissionStatus::Denied,
                    message: "Camera access denied by user".to_string(),
                    can_request: false,
                })
            }
            Err(_) => {
                log::error!("Permission request timed out");
                Err("Permission request timed out".to_string())
            }
        }
    }
}

/// Check camera permission status
#[command]
pub async fn check_camera_permission_status() -> Result<PermissionInfo, String> {
    log::debug!("Checking camera permission status");
    Ok(check_permission_detailed())
}

/// Get human-readable permission status string (legacy compatibility)
#[command]
pub fn get_permission_status_string() -> String {
    let info = check_permission_detailed();
    format!("{:?}", info.status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    async fn test_check_permission_status() {
        let result = check_camera_permission_status().await;
        assert!(result.is_ok());

        let info = result.unwrap();
        println!("Permission status: {:?}", info.status);
        println!("Message: {}", info.message);
    }

    #[test]
    #[ignore = "Requires camera hardware and OS permissions - run manually"]
    fn test_permission_status_string() {
        let status = get_permission_status_string();
        assert!(!status.is_empty());
        println!("Status string: {}", status);
    }
}
