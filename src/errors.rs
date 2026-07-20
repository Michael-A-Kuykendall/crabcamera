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
    /// System resource or access error.
    AccessError(String),
    /// Connection implementation error.
    ConnectionError(String),
    /// Internal system error.
    SystemError(String),
    /// Invalid configuration.
    ConfigError(String),
}

impl fmt::Display for CameraError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CameraError::InitializationError(msg) => {
                write!(f, "Camera initialization error: {msg}")
            }
            CameraError::PermissionDenied(msg) => write!(f, "Permission denied error: {msg}"),
            CameraError::CaptureError(msg) => write!(f, "Capture error: {msg}"),
            CameraError::ControlError(msg) => write!(f, "Camera control error: {msg}"),
            CameraError::StreamError(msg) => write!(f, "Stream error: {msg}"),
            CameraError::UnsupportedOperation(msg) => write!(f, "Unsupported operation: {msg}"),
            #[cfg(feature = "recording")]
            CameraError::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
            #[cfg(feature = "recording")]
            CameraError::MuxingError(msg) => write!(f, "Muxing error: {msg}"),
            #[cfg(feature = "recording")]
            CameraError::IoError(msg) => write!(f, "IO error: {msg}"),
            #[cfg(feature = "audio")]
            CameraError::AudioError(msg) => write!(f, "Audio error: {msg}"),
            CameraError::AccessError(msg) => write!(f, "Access error: {msg}"),
            CameraError::ConnectionError(msg) => write!(f, "Connection error: {msg}"),
            CameraError::SystemError(msg) => write!(f, "System error: {msg}"),
            CameraError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
        }
    }
}

impl From<CameraError> for String {
    fn from(err: CameraError) -> Self {
        err.to_string()
    }
}

impl std::error::Error for CameraError {}

#[cfg(test)]
mod tests {
    use super::CameraError;

    #[test]
    fn test_display_messages_for_all_core_variants() {
        let cases = vec![
            (CameraError::InitializationError("init".to_string()), "Camera initialization error: init"),
            (CameraError::PermissionDenied("perm".to_string()), "Permission denied error: perm"),
            (CameraError::CaptureError("capture".to_string()), "Capture error: capture"),
            (CameraError::ControlError("control".to_string()), "Camera control error: control"),
            (CameraError::StreamError("stream".to_string()), "Stream error: stream"),
            (CameraError::UnsupportedOperation("unsupported".to_string()), "Unsupported operation: unsupported"),
            (CameraError::AccessError("access".to_string()), "Access error: access"),
            (CameraError::ConnectionError("connection".to_string()), "Connection error: connection"),
            (CameraError::SystemError("system".to_string()), "System error: system"),
            (CameraError::ConfigError("config".to_string()), "Configuration error: config"),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[cfg(feature = "recording")]
    #[test]
    fn test_display_messages_for_recording_variants() {
        let cases = vec![
            (CameraError::EncodingError("enc".to_string()), "Encoding error: enc"),
            (CameraError::MuxingError("mux".to_string()), "Muxing error: mux"),
            (CameraError::IoError("io".to_string()), "IO error: io"),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_display_message_for_audio_variant() {
        let error = CameraError::AudioError("audio".to_string());
        assert_eq!(error.to_string(), "Audio error: audio");
    }

    #[test]
    fn test_into_string_and_error_trait() {
        let error = CameraError::CaptureError("boom".to_string());
        let as_string: String = error.into();
        assert_eq!(as_string, "Capture error: boom");

        let err_obj: &dyn std::error::Error = &CameraError::SystemError("x".to_string());
        assert!(err_obj.source().is_none());
    }
}
