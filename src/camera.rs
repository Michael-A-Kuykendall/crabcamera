/// Legacy camera structure - kept for backwards compatibility
/// Use PlatformCamera for actual camera operations
#[derive(Default)]
pub struct Camera {
    _private: (),
}

impl Camera {
    /// Create a new legacy camera instance
    /// Note: This is deprecated. Use `PlatformCamera::new()` instead
    #[deprecated(note = "Use PlatformCamera::new() from the platform module")]
    pub fn new() -> Self {
        Self { _private: () }
    }
}