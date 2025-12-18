use std::fmt;

#[derive(Debug)]
pub enum CameraError {
    InitializationError(String),
    PermissionDenied(String),
    CaptureError(String),
    ControlError(String),
    StreamError(String),
    #[cfg(feature = "recording")]
    EncodingError(String),
    #[cfg(feature = "recording")]
    MuxingError(String),
    #[cfg(feature = "recording")]
    IoError(String),
    #[cfg(feature = "audio")]
    AudioError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::InitializationError(msg) => write!(f, "Camera initialization error: {}", msg),
            CameraError::PermissionDenied(msg) => write!(f, "Permission denied error: {}", msg),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            CameraError::ControlError(msg) => write!(f, "Camera control error: {}", msg),
            CameraError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            #[cfg(feature = "recording")]
            CameraError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            #[cfg(feature = "recording")]
            CameraError::MuxingError(msg) => write!(f, "Muxing error: {}", msg),
            #[cfg(feature = "recording")]
            CameraError::IoError(msg) => write!(f, "IO error: {}", msg),
            #[cfg(feature = "audio")]
            CameraError::AudioError(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}

impl std::error::Error for CameraError {}