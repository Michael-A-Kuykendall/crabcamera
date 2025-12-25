use crate::errors::CameraError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadlessErrorKind {
    Timeout,
    Closed,
    Stopped,
    AlreadyStarted,
    AlreadyStopped,
    AlreadyClosed,
    NotFound,
    InvalidArgument,
    Unsupported,
    Backend,
    PoisonedLock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessError {
    pub kind: HeadlessErrorKind,
    pub message: String,
}

impl HeadlessError {
    pub fn timeout() -> Self {
        Self {
            kind: HeadlessErrorKind::Timeout,
            message: "timeout".to_string(),
        }
    }

    pub fn closed() -> Self {
        Self {
            kind: HeadlessErrorKind::Closed,
            message: "session is closed".to_string(),
        }
    }

    pub fn stopped() -> Self {
        Self {
            kind: HeadlessErrorKind::Stopped,
            message: "session is stopped".to_string(),
        }
    }

    pub fn already_started() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyStarted,
            message: "session is already started".to_string(),
        }
    }

    pub fn already_stopped() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyStopped,
            message: "session is already stopped".to_string(),
        }
    }

    pub fn already_closed() -> Self {
        Self {
            kind: HeadlessErrorKind::AlreadyClosed,
            message: "session is already closed".to_string(),
        }
    }

    pub fn not_found(entity: &str, id: &str) -> Self {
        Self {
            kind: HeadlessErrorKind::NotFound,
            message: format!("{entity} not found: {id}"),
        }
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            kind: HeadlessErrorKind::InvalidArgument,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: HeadlessErrorKind::Unsupported,
            message: message.into(),
        }
    }

    pub fn backend(error: CameraError) -> Self {
        Self {
            kind: HeadlessErrorKind::Backend,
            message: error.to_string(),
        }
    }

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
