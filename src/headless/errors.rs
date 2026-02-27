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
