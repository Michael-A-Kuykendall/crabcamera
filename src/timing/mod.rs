//! Basic timing utilities for presentation timestamps
//!
//! Simple monotonic clock for timestamp generation.

use std::sync::Arc;
use std::time::Instant;

/// Monotonic clock for presentation timestamps
///
/// All timestamps derive from this single source
/// to ensure monotonic ordering.
#[derive(Debug, Clone)]
pub struct PTSClock {
    start: Arc<Instant>,
}

impl PTSClock {
    /// Create a new PTS clock with the current instant as time zero
    pub fn new() -> Self {
        Self {
            start: Arc::new(Instant::now()),
        }
    }

    /// Create a PTS clock from an existing start instant
    ///
    /// Use this to share the same timebase between components.
    pub fn from_instant(start: Instant) -> Self {
        Self {
            start: Arc::new(start),
        }
    }

    /// Get the presentation timestamp in seconds
    ///
    /// Returns the elapsed time since clock creation.
    #[inline]
    pub fn pts(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Get the presentation timestamp for a given instant
    ///
    /// The instant must be after the clock's start time.
    #[inline]
    pub fn pts_at(&self, instant: Instant) -> f64 {
        instant.duration_since(*self.start).as_secs_f64()
    }

    /// Get the start instant for sharing with other components
    pub fn start_instant(&self) -> Instant {
        *self.start
    }
}

impl Default for PTSClock {
    fn default() -> Self {
        Self::new()
    }
}