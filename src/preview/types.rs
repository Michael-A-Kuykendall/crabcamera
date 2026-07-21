use crate::quality::QualityReport;
use serde::Serialize;

/// Event emitted by `PreviewStream` for each captured frame.
/// Carries a JPEG-compressed preview frame alongside quality metadata.
#[derive(Debug, Clone, Serialize)]
pub struct PreviewFrameEvent {
    /// JPEG-compressed frame data (Vec<u8> for Tauri serialization)
    pub jpeg_data: Vec<u8>,
    /// Quality report from `SmartTrigger`. None = still analyzing first frames.
    pub quality: Option<QualityReport>,
    /// True when the quality report was sampled from a prior frame, not the current one.
    pub stale: bool,
    /// Which frame number the (possibly stale) quality data came from.
    pub last_sampled_frame: u64,
    /// `SmartTrigger` reports Ready when conditions are stable for capture.
    pub is_smart_trigger_ready: bool,
    /// Monotonic UTC timestamp of frame capture.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Monotonically increasing frame counter.
    pub frame_number: u64,
}

/// Configuration for a `PreviewStream` session.
#[derive(Debug, Clone)]
pub struct PreviewConfig {
    /// Target frames per second (1-60).
    pub fps_target: u32,
    /// Downscale factor for the RGB preview (0.1-1.0).
    /// 1.0 = no RGB resize; control wire size via `jpeg_quality` instead.
    pub downscale: f32,
    /// Run quality analysis on every Nth frame (1 = every frame).
    pub quality_sample_rate: u32,
    /// If true, quality analysis uses the full-resolution frame even when downscale < 1.0.
    /// If false, quality runs on the downscaled preview (faster, slightly less accurate).
    pub analyze_at_full_res: bool,
    /// JPEG quality 30-95. Lower = smaller payload, less CPU.
    pub jpeg_quality: u8,
}

impl PreviewConfig {
    /// Validate that all config fields are within acceptable bounds.
    ///
    /// # Errors
    /// Returns an `Err` describing the first out-of-range field if
    /// `fps_target`, `downscale`, `quality_sample_rate`, or `jpeg_quality`
    /// falls outside its allowed range.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=60).contains(&self.fps_target) {
            return Err("fps_target must be 1-60".into());
        }
        if !(0.1..=1.0).contains(&self.downscale) {
            return Err("downscale must be 0.1-1.0".into());
        }
        if self.quality_sample_rate == 0 {
            return Err("quality_sample_rate must be >= 1".into());
        }
        if !(30..=95).contains(&self.jpeg_quality) {
            return Err("jpeg_quality must be 30-95".into());
        }
        Ok(())
    }
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            fps_target: 15,
            downscale: 0.5,
            quality_sample_rate: 5,
            analyze_at_full_res: false,
            jpeg_quality: 70,
        }
    }
}
