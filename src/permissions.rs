/// Permission status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionStatus {
    /// Permission granted
    Granted,
    /// Permission denied
    Denied,
    /// Permission not determined (user hasn't been asked yet)
    NotDetermined,
    /// Permission restricted (parental controls, etc)
    Restricted,
}

impl std::fmt::Display for PermissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionStatus::Granted => write!(f, "granted"),
            PermissionStatus::Denied => write!(f, "denied"),
            PermissionStatus::NotDetermined => write!(f, "not_determined"),
            PermissionStatus::Restricted => write!(f, "restricted"),
        }
    }
}

/// Check camera permission status
/// Returns permission status for the current platform
pub fn check_permission() -> PermissionStatus {
    check_permission_detailed().status
}

/// Check camera permission status with detailed information
pub fn check_permission_detailed() -> PermissionInfo {
    #[cfg(target_os = "windows")]
    {
        check_permission_windows()
    }

    #[cfg(target_os = "macos")]
    {
        check_permission_macos()
    }

    #[cfg(target_os = "linux")]
    {
        check_permission_linux()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PermissionInfo {
            status: PermissionStatus::NotDetermined,
            message: "Platform not supported".to_string(),
            can_request: false,
        }
    }
}

/// Detailed permission information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionInfo {
    pub status: PermissionStatus,
    pub message: String,
    pub can_request: bool,
}

#[cfg(target_os = "windows")]
fn check_permission_windows() -> PermissionInfo {
    // On Windows 10+, camera access is controlled by Privacy settings
    // Check if we can enumerate devices as a proxy for permission
    use nokhwa::query;

    match query(nokhwa::utils::ApiBackend::Auto) {
        Ok(devices) if !devices.is_empty() => PermissionInfo {
            status: PermissionStatus::Granted,
            message: "Camera access granted via Windows Privacy settings".to_string(),
            can_request: false,
        },
        Ok(_) => PermissionInfo {
            status: PermissionStatus::NotDetermined,
            message: "No cameras found - permission may not be granted".to_string(),
            can_request: true,
        },
        Err(e) => PermissionInfo {
            status: PermissionStatus::Denied,
            message: format!("Camera access denied: {}", e),
            can_request: true,
        },
    }
}

#[cfg(target_os = "macos")]
fn check_permission_macos() -> PermissionInfo {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;

    unsafe {
        // Get AVCaptureDevice class
        let av_capture_device_class = Class::get("AVCaptureDevice");

        if av_capture_device_class.is_none() {
            return PermissionInfo {
                status: PermissionStatus::NotDetermined,
                message: "AVFoundation not available".to_string(),
                can_request: false,
            };
        }

        let av_capture_device_class = av_capture_device_class.unwrap();

        // Get media type for video
        let av_media_type_video = CString::new("vide").unwrap();
        let media_type: *mut Object =
            msg_send![av_capture_device_class, mediaTypeForString: av_media_type_video.as_ptr()];

        // Check authorization status
        let auth_status: i64 =
            msg_send![av_capture_device_class, authorizationStatusForMediaType: media_type];

        // AVAuthorizationStatus enum values:
        // 0 = NotDetermined
        // 1 = Restricted
        // 2 = Denied
        // 3 = Authorized

        match auth_status {
            3 => PermissionInfo {
                status: PermissionStatus::Granted,
                message: "Camera access authorized".to_string(),
                can_request: false,
            },
            2 => PermissionInfo {
                status: PermissionStatus::Denied,
                message: "Camera access denied - enable in System Preferences > Security & Privacy > Camera".to_string(),
                can_request: false,
            },
            1 => PermissionInfo {
                status: PermissionStatus::Restricted,
                message: "Camera access restricted by system policy".to_string(),
                can_request: false,
            },
            _ => PermissionInfo {
                status: PermissionStatus::NotDetermined,
                message: "Camera permission not yet requested".to_string(),
                can_request: true,
            },
        }
    }
}

#[cfg(target_os = "linux")]
fn check_permission_linux() -> PermissionInfo {
    use std::fs;
    use std::path::Path;

    // Check if any video devices exist
    let video_devices: Vec<_> = (0..10)
        .map(|i| format!("/dev/video{}", i))
        .filter(|path| Path::new(path).exists())
        .collect();

    if video_devices.is_empty() {
        return PermissionInfo {
            status: PermissionStatus::NotDetermined,
            message: "No video devices found at /dev/video*".to_string(),
            can_request: false,
        };
    }

    // Check if we can read from first video device
    let first_device = &video_devices[0];
    match fs::metadata(first_device) {
        Ok(_metadata) => {
            // Check if we have read permission (via group membership)
            if check_linux_group_membership() {
                PermissionInfo {
                    status: PermissionStatus::Granted,
                    message: format!(
                        "Camera access granted (user in video group, {} found)",
                        first_device
                    ),
                    can_request: false,
                }
            } else {
                PermissionInfo {
                    status: PermissionStatus::Denied,
                    message: format!("Camera device {} exists but user not in video group - run: sudo usermod -a -G video $USER", first_device),
                    can_request: true,
                }
            }
        }
        Err(e) => PermissionInfo {
            status: PermissionStatus::Denied,
            message: format!("Cannot access {}: {}", first_device, e),
            can_request: true,
        },
    }
}

#[cfg(target_os = "linux")]
fn check_linux_group_membership() -> bool {
    use std::process::Command;

    // Check if user is in 'video' or 'plugdev' group
    let output = Command::new("groups").output().ok();

    if let Some(output) = output {
        if let Ok(groups) = String::from_utf8(output.stdout) {
            return groups.contains("video") || groups.contains("plugdev");
        }
    }

    // Fallback: assume permission if we can't check groups
    false
}
