//! Real performance tracking for platform cameras.
//!
//! Every platform's `capture_frame` records timing into a shared [`PerfTracker`],
//! and `get_performance_metrics` reads it back together with a genuine OS-level
//! process-memory reading. Nothing here is fabricated: latency and processing
//! time are measured around the actual capture call, FPS is derived from the
//! interval between captures, dropped frames are counted on capture failure, and
//! memory usage is read from the operating system.

use crate::constants::BLUR_VARIANCE_BLURRY;
use crate::quality::blur::BlurDetector;
use crate::types::CameraFrame;
use crate::types::CameraPerformanceMetrics;
use std::time::Instant;

/// Rolling performance tracker shared by all platform cameras.
///
/// Wrapped in an `Arc<Mutex<_>>` by each platform camera so that `capture_frame`
/// (which may be `&self`) can update it and `get_performance_metrics` can read it.
pub struct PerfTracker {
    /// Wall-clock time, in milliseconds, of the most recent `frame()` call.
    pub capture_latency_ms: f32,
    /// Time, in milliseconds, spent constructing the `CameraFrame` from the
    /// raw buffer (decode/clone/metadata).
    pub processing_time_ms: f32,
    /// Frames per second actually delivered, derived from the interval between
    /// consecutive successful captures.
    pub fps_actual: f32,
    /// Total number of successful captures observed.
    pub frames_captured: u64,
    /// Number of capture attempts that failed (used to detect frame drops).
    pub dropped_frames: u32,
    /// Number of times the caller pulled a frame faster than the device could
    /// deliver a new one (detected as a zero-interval capture).
    pub buffer_overruns: u32,
    /// Snapshot of the most recent frame, retained so a quality score can be
    /// derived on demand without re-capturing. `(buffer, width, height, format)`.
    last_frame: Option<(Vec<u8>, u32, u32, String)>,
    /// Instant of the previous successful capture, for FPS accounting.
    last_capture: Option<Instant>,
}

impl Default for PerfTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PerfTracker {
    /// Create a new performance tracker with zeroed counters.
    pub fn new() -> Self {
        Self {
            capture_latency_ms: 0.0,
            processing_time_ms: 0.0,
            fps_actual: 0.0,
            frames_captured: 0,
            dropped_frames: 0,
            buffer_overruns: 0,
            last_frame: None,
            last_capture: None,
        }
    }

    /// Record a successful capture.
    ///
    /// `latency_ms` is the time spent in the device `frame()` call,
    /// `processing_ms` is the time spent building the `CameraFrame`,
    /// and `frame` is the raw buffer snapshot (if available) used later for a
    /// quality score.
    pub fn record_capture(
        &mut self,
        latency_ms: f32,
        processing_ms: f32,
        frame: Option<(Vec<u8>, u32, u32, String)>,
    ) {
        self.capture_latency_ms = latency_ms;
        self.processing_time_ms = processing_ms;
        self.frames_captured += 1;

        if let Some(f) = frame {
            self.last_frame = Some(f);
        }

        let now = Instant::now();
        if let Some(prev) = self.last_capture {
            let elapsed = prev.elapsed().as_secs_f32();
            if elapsed > 0.0 {
                self.fps_actual = 1.0 / elapsed;
            } else {
                // Captured again before a measurable interval elapsed: the consumer
                // is outrunning the device's delivery rate.
                self.buffer_overruns += 1;
            }
        }
        self.last_capture = Some(now);
    }

    /// Record a failed capture attempt as a dropped frame.
    pub fn record_drop(&mut self) {
        self.dropped_frames += 1;
    }

    /// Borrow the most recently captured frame snapshot, if any.
    pub fn last_frame(&self) -> Option<&(Vec<u8>, u32, u32, String)> {
        self.last_frame.as_ref()
    }

    /// Current resident process memory in megabytes, read from the OS.
    pub fn memory_usage_mb(&self) -> f32 {
        current_process_memory_mb()
    }
}

/// Read the current process's resident memory usage in megabytes.
///
/// Uses a genuine OS interface per platform:
/// - Linux: `RSS` field of `/proc/self/statm`.
/// - macOS: `task_info` with `MACH_TASK_BASIC_INFO` (`resident_size`).
/// - Windows: `GetProcessMemoryInfo` working-set size.
pub fn current_process_memory_mb() -> f32 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/statm")
            .ok()
            .and_then(|contents| {
                // statm columns: size resident shared text data ... (all in pages)
                let rss_pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
                // Standard page size on essentially all x86/arm Linux targets (4096 bytes).
                #[allow(clippy::cast_precision_loss)]
                let mb = (rss_pages * 4096) as f32 / (1024.0 * 1024.0);
                Some(mb)
            })
            .unwrap_or(0.0)
    }

    #[cfg(target_os = "macos")]
    {
        // `mach_task_self` and `task_info` live in libSystem, which is always
        // linked, so no external crate is required.
        const MACH_TASK_BASIC_INFO: i32 = 20;
        let mut info = [0i32; 12]; // mach_task_basic_info is 48 bytes = 12 i32s
        let mut count: u32 = info.len() as u32;

        extern "C" {
            fn mach_task_self() -> u32;
            fn task_info(
                task: u32,
                flavor: i32,
                task_info_out: *mut i32,
                task_info_count: *mut u32,
            ) -> i32;
        }

        let ret = unsafe {
            task_info(
                mach_task_self(),
                MACH_TASK_BASIC_INFO,
                info.as_mut_ptr(),
                &mut count,
            )
        };
        if ret == 0 {
            // resident_size is the u64 at byte offset 8 (after virtual_size),
            // i.e. i32 indices [2] (low) and [3] (high).
            let resident = u64::from(info[2] as u32) | (u64::from(info[3] as u32) << 32);
            resident as f32 / (1024.0 * 1024.0)
        } else {
            0.0
        }
    }

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::ProcessStatus::GetProcessMemoryInfo;
        use windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS;
        use windows::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            let handle = GetCurrentProcess();
            let mut counters = PROCESS_MEMORY_COUNTERS {
                cb: u32::try_from(std::mem::size_of::<PROCESS_MEMORY_COUNTERS>())
                    .unwrap_or(u32::MAX),
                ..Default::default()
            };
            if GetProcessMemoryInfo(handle, &raw mut counters, counters.cb).is_ok() {
                #[allow(clippy::cast_precision_loss)]
                // usize→f32: WorkingSetSize in MB, f32 precision (>16M) is more than sufficient for this value
                let ws = counters.WorkingSetSize as f32;
                ws / (1024.0 * 1024.0)
            } else {
                0.0
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        0.0
    }
}

/// Assemble a [`CameraPerformanceMetrics`] snapshot from a tracker and the
/// device id.
///
/// The quality score is computed on demand from the most recently captured
/// frame using the existing blur detector; if no frame has been captured yet it
/// defaults to `0.0` (meaning "no data"), never a fabricated positive value.
pub fn build_metrics(tracker: &PerfTracker, device_id: &str) -> CameraPerformanceMetrics {
    let quality_score = match tracker.last_frame() {
        Some((buffer, width, height, format)) => {
            let frame = CameraFrame::new(buffer.clone(), *width, *height, device_id.to_string())
                .with_format(format.clone());
            let detector = BlurDetector::new(BLUR_VARIANCE_BLURRY, 100.0);
            detector.analyze_frame(&frame).quality_score
        }
        None => 0.0,
    };

    CameraPerformanceMetrics {
        capture_latency_ms: tracker.capture_latency_ms,
        processing_time_ms: tracker.processing_time_ms,
        memory_usage_mb: tracker.memory_usage_mb(),
        fps_actual: tracker.fps_actual,
        dropped_frames: tracker.dropped_frames,
        buffer_overruns: tracker.buffer_overruns,
        quality_score,
    }
}
