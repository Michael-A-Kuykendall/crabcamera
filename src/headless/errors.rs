use crate::errors::CameraError;

/// The simplified category of error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadlessErrorKind {
    /// Operation timed out.
    Timeout,
    /// Session was closed.
    Closed,
    /// Session is stopped but expected to be running.
    Stopped,
    /// Session already in started state.
    AlreadyStarted,
    /// Session already in stopped state.
    AlreadyStopped,
    /// Session already in closed state.
    AlreadyClosed,
    /// Requested resource definition not found.
    NotFound,
    /// Invalid input parameter.
    InvalidArgument,
    /// Feature not supported in current configuration.
    Unsupported,
    /// Underlying camera backend error.
    Backend,
    /// Thread sync primitive poisoned.
    PoisonedLock,
}

/// A structured error type for headless operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessError {
    /// The high-level category of this error.
    pub kind: HeadlessErrorKind,
    /// A human-readable description of the error.
    pub message: String,
}

impl HeadlessError {
    /// Creates a timeout error.
    #[must_use]
    pub fn timeout() -> Self {
        Self {
            kind: HeadlessErrorKind::Timeout,
            message: "timeout".to_string(),
        }
    }

    /// Creates a closed session error.
    #[must_use]
    pub fn closed() -> Self {
        Self {
            kind: HeadlessErrorKind::Closed,
            message: "session is closed".to_string(),
        }
    }

    /// Creates a stopped session error.
    #[must_use]
    pub fn stopped() -> Self {
        Self {
            kind: HeadlessErrorKind::Stopped,
            message: "session is stopped".to_string(),
        }
    }

    /// Creates an already started error.
    #[must_use]
    pub fn already_started() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyStarted,
            message: "session is already started".to_string(),
        }
    }

    /// Creates an already stopped error.
    #[must_use]
    pub fn already_stopped() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyStopped,
            message: "session is already stopped".to_string(),
        }
    }

    /// Creates an already closed error.
    #[must_use]
    pub fn already_closed() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyClosed,
            message: "session is already closed".to_string(),
        }
    }

    /// Creates a not found error.
    #[must_use]
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self {
            kind: HeadlessErrorKind::NotFound,
            message: format!("{entity} not found: {id}"),
        }
    }

    /// Creates an invalid argument error.
    #[must_use]
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            kind: HeadlessErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    /// Creates an unsupported feature error.
    #[must_use]
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: HeadlessErrorKind::Unsupported,
            message: message.into(),
        }
    }

    /// Wraps a backend camera error.
    #[must_use]
    pub fn backend(error: CameraError) -> Self {
        Self {
            kind: HeadlessErrorKind::Backend,
            message: error.to_string(),
        }
    }

    /// Creates a lock poisoned error.
    #[must_use]
    pub fn poisoned_lock() -> Self {
        Self {
            kind: HeadlessErrorKind::PoisonedLock,
            message: "lock poisoned by previous panic".to_string(),
        }
    }
}

impl std::fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HeadlessError {}

#[cfg(test)]
mod tests {
    use super::{HeadlessError, HeadlessErrorKind};
    use crate::errors::CameraError;

    #[test]
    fn test_error_constructors_set_expected_kind_and_message() {
        let cases = vec![
            (HeadlessError::timeout(), HeadlessErrorKind::Timeout, "timeout"),
            (HeadlessError::closed(), HeadlessErrorKind::Closed, "session is closed"),
            (HeadlessError::stopped(), HeadlessErrorKind::Stopped, "session is stopped"),
            (
                HeadlessError::already_started(),
                HeadlessErrorKind::AlreadyStarted,
                "session is already started",
            ),
            (
                HeadlessError::already_stopped(),
                HeadlessErrorKind::AlreadyStopped,
                "session is already stopped",
            ),
            (
                HeadlessError::already_closed(),
                HeadlessErrorKind::AlreadyClosed,
                "session is already closed",
            ),
            (
                HeadlessError::poisoned_lock(),
                HeadlessErrorKind::PoisonedLock,
                "lock poisoned by previous panic",
            ),
        ];

        for (err, kind, msg) in cases {
            assert_eq!(err.kind, kind);
            assert_eq!(err.message, msg);
            assert_eq!(err.to_string(), msg);
        }
    }

    #[test]
    fn test_not_found_and_invalid_argument_and_unsupported() {
        let not_found = HeadlessError::not_found("device", "42");
        assert_eq!(not_found.kind, HeadlessErrorKind::NotFound);
        assert_eq!(not_found.message, "device not found: 42");

        let invalid = HeadlessError::invalid_argument("bad value");
        assert_eq!(invalid.kind, HeadlessErrorKind::InvalidArgument);
        assert_eq!(invalid.message, "bad value");

        let unsupported = HeadlessError::unsupported("feature disabled");
        assert_eq!(unsupported.kind, HeadlessErrorKind::Unsupported);
        assert_eq!(unsupported.message, "feature disabled");
    }

    #[test]
    fn test_backend_wraps_camera_error() {
        let backend = HeadlessError::backend(CameraError::CaptureError("camera exploded".to_string()));
        assert_eq!(backend.kind, HeadlessErrorKind::Backend);
        assert!(backend.message.contains("Capture error"));
        assert!(backend.message.contains("camera exploded"));
    }

    #[test]
    fn test_error_trait_impl() {
        let err = HeadlessError::timeout();
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }
}
