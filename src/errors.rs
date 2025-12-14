use std::fmt;

#[derive(Debug)]
pub enum CameraError {
    InitializationError(String),
    PermissionDenied(String),
    CaptureError(String),
    ControlError(String),
    StreamError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::InitializationError(msg) => write!(f, "Camera initialization error: {}", msg),
            CameraError::PermissionDenied(msg) => write!(f, "Permission denied error: {}", msg),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            CameraError::ControlError(msg) => write!(f, "Camera control error: {}", msg),
            CameraError::StreamError(msg) => write!(f, "Stream error: {}", msg),
        }
    }
}

impl std::error::Error for CameraError {}