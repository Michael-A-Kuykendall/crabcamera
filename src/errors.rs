use std::fmt;

/// The top-level error type for camera operations.
#[derive(Debug)]
pub enum CameraError {
    /// Failed to initialize the camera backend or device.
    InitializationError(String),
    /// Permission denied by OS or user.
    PermissionDenied(String),
    /// Failed to capture a frame.
    CaptureError(String),
    /// Failed to set a camera control.
    ControlError(String),
    /// Error in the video stream pipeline.
    StreamError(String),
    /// Operation not supported by the current hardware or platform.
    UnsupportedOperation(String),
    #[cfg(feature = "recording")]
    /// Video encoding initialization or processing error.
    EncodingError(String),
    #[cfg(feature = "recording")]
    /// Container muxing error.
    MuxingError(String),
    #[cfg(feature = "recording")]
    /// File system I/O error during recording.
    IoError(String),
    #[cfg(feature = "audio")]
    /// Audio device or capture error.
    AudioError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::InitializationError(msg) => {
                write!(f, "Camera initialization error: {}", msg)
            }
            CameraError::PermissionDenied(msg) => write!(f, "Permission denied error: {}", msg),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {}", msg),
            CameraError::ControlError(msg) => write!(f, "Camera control error: {}", msg),
            CameraError::StreamError(msg) => write!(f, "Stream error: {}", msg),
            CameraError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {}", msg),
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
