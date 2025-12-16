use std::fmt;

#[derive(Debug)]
pub enum CameraError {
    InitializationError(String),
    PermissionDenied(String),
    CaptureError(String),
    ControlError(String),
    StreamError(String),
    /// Video encoding error (H.264)
    #[cfg(feature = "recording")]
    EncodingError(String),
    /// MP4 muxing error
    #[cfg(feature = "recording")]
    MuxingError(String),
    /// General I/O error
    #[cfg(feature = "recording")]
    IoError(String),
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
            CameraError::EncodingError(msg) => write!(f, "Video encoding error: {}", msg),
            #[cfg(feature = "recording")]
            CameraError::MuxingError(msg) => write!(f, "MP4 muxing error: {}", msg),
            #[cfg(feature = "recording")]
            CameraError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CameraError {}