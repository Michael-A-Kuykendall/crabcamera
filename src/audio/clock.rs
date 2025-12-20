//! Presentation timestamp clock for A/V synchronization
//!
//! Defines a single monotonic timebase for all audio and video timestamps,
//! ensuring synchronized audio/video playback with bounded drift (±40ms max).
//!
//! ## Properties
//!
//! - Monotonic: timestamps never decrease
//! - Non-decreasing: maintains strict ordering
//! - Shared: used by both audio and video streams
//! - Platform-independent: uses `std::time::Instant`

use std::sync::Arc;
use std::time::Instant;

/// Monotonic clock for presentation timestamps
///
/// All audio and video timestamps derive from this single source
/// to ensure bounded sync drift.
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
    /// Use this to share the same timebase between video and audio.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_pts_monotonic() {
        let clock = PTSClock::new();
        let pts1 = clock.pts();
        thread::sleep(Duration::from_millis(10));
        let pts2 = clock.pts();
        assert!(pts2 > pts1, "PTS must be monotonically increasing");
    }

    #[test]
    fn test_pts_non_negative() {
        let clock = PTSClock::new();
        assert!(clock.pts() >= 0.0);
    }

    #[test]
    fn test_shared_clock() {
        let clock1 = PTSClock::new();
        let clock2 = PTSClock::from_instant(clock1.start_instant());

        thread::sleep(Duration::from_millis(5));

        let pts1 = clock1.pts();
        let pts2 = clock2.pts();

        // Should be very close (within 1ms)
        assert!((pts1 - pts2).abs() < 0.001);
    }

    #[test]
    fn test_pts_at() {
        let start = Instant::now();
        let clock = PTSClock::from_instant(start);

        thread::sleep(Duration::from_millis(10));
        let now = Instant::now();

        let pts = clock.pts_at(now);
        assert!(pts >= 0.01, "Should be at least 10ms");
    }
}
