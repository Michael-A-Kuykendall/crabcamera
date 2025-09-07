use std::fmt;

#[derive(Debug)]
pub enum CameraError {
    InitializationError(String),
    PermissionDenied(String),
    CaptureError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::InitializationError(msg) => write!(f, "Camera initialization error: {}", msg),
            CameraError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
        }
    }
}

impl std::error::Error for CameraError {}