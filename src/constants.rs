//! Global constants for CrabCamera
//!
//! This file centralizes all hardcoded values to prevent "magic number" usage
//! and ensure consistency across the application.

/// Default camera resolution width (Full HD)
pub const DEFAULT_RESOLUTION_WIDTH: u32 = 1920;

/// Default camera resolution height (Full HD)
pub const DEFAULT_RESOLUTION_HEIGHT: u32 = 1080;

/// Fallback resolution width (HD Ready)
pub const FALLBACK_RESOLUTION_WIDTH: u32 = 1280;

/// Fallback resolution height (HD Ready)
pub const FALLBACK_RESOLUTION_HEIGHT: u32 = 720;

/// Minimal resolution width (VGA)
pub const MIN_RESOLUTION_WIDTH: u32 = 640;

/// Minimal resolution height (VGA)
pub const MIN_RESOLUTION_HEIGHT: u32 = 480;

/// Maximum resolution width (4K)
pub const MAX_RESOLUTION_WIDTH: u32 = 3840;

/// Maximum resolution height (4K)
pub const MAX_RESOLUTION_HEIGHT: u32 = 2160;

/// Default frame rate
pub const DEFAULT_FPS: f32 = 30.0;

/// High frame rate
pub const HIGH_FPS: f32 = 60.0;

/// Linux video device prefix
pub const LINUX_VIDEO_DEVICE_PREFIX: &str = "/dev/video";

/// Default ISO sensitivity
pub const DEFAULT_ISO: u32 = 400;

/// Minimum ISO sensitivity
pub const MIN_ISO: u32 = 50;

/// Maximum ISO sensitivity
pub const MAX_ISO: u32 = 12800;

/// Default video format type
pub const DEFAULT_FORMAT_TYPE: &str = "YUYV";

/// RGB format type
pub const FORMAT_RGB: &str = "RGB8";

/// MJPEG format type
pub const FORMAT_MJPEG: &str = "MJPEG";

/// Default frame pool size
pub const DEFAULT_POOL_SIZE: usize = 10;

/// Default bytes per pixel (RGB8)
pub const BYTES_PER_PIXEL_RGB: u32 = 3;

/// Default Reconnect Attempts
pub const DEFAULT_RECONNECT_ATTEMPTS: u32 = 3;

/// Default Reconnect Delay (ms)
pub const DEFAULT_RECONNECT_DELAY_MS: u64 = 1000;

/// Default Max Retry Attempts
pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 10;

/// Default Blur Threshold (0.0-1.0)
pub const DEFAULT_BLUR_THRESHOLD: f32 = 0.7;

/// Default Exposure Score Threshold (0.0-1.0)
pub const DEFAULT_EXPOSURE_THRESHOLD: f32 = 0.6;

/// Default Overall Quality Score Threshold (0.0-1.0)
pub const DEFAULT_OVERALL_THRESHOLD: f32 = 0.7;

/// Default Retry Delay (ms)
pub const DEFAULT_RETRY_DELAY_MS: u64 = 100;

/// Default Output Directory
pub const DEFAULT_OUTPUT_DIRECTORY: &str = "./captures";

/// Default Date Format
pub const DEFAULT_DATE_FORMAT: &str = "YYYY-MM-DD";

/// Default Image Format
pub const DEFAULT_IMAGE_FORMAT: &str = "jpeg";

/// Default JPEG Quality (0-100)
pub const DEFAULT_JPEG_QUALITY: u8 = 95;

/// Default Focus Stack Steps
pub const DEFAULT_FOCUS_STACK_STEPS: u32 = 10;

/// Default HDR Brackets
pub const DEFAULT_HDR_BRACKETS: u32 = 3;

/// Audio sample rate (Standard Opus requirement)
pub const AUDIO_SAMPLE_RATE: u32 = 48000;

/// Audio channels (Stereo)
pub const AUDIO_CHANNELS: u16 = 2;

/// Audio bitrate (Standard quality)
pub const AUDIO_BITRATE: u32 = 128_000;

/// Audio Capture
pub const AUDIO_SAMPLE_RATE_48K: u32 = 48000;
/// Audio Capture - 44.1kHz Sample Rate
pub const AUDIO_SAMPLE_RATE_44K: u32 = 44100;
/// Audio Capture - Buffer Size in Frames
pub const AUDIO_BUFFER_FRAMES: usize = 256;
/// Audio Capture - Default Device ID
pub const AUDIO_DEVICE_DEFAULT: &str = "default";
/// Audio Capture - Default Bitrate (128kbps)
pub const AUDIO_DEFAULT_BITRATE: u32 = 128_000;

/// CLI Defaults
/// Default timeout for capture operations in ms
pub const CLI_DEFAULT_TIMEOUT_MS: u64 = 1000;
/// Default number of frames to capture
pub const CLI_DEFAULT_FRAME_COUNT: usize = 1;
/// Identifier for file-based sources in CLI
pub const CLI_FILE_SOURCE_ID: &str = "cli-file-source";
/// Exit code for errors
pub const CLI_EXIT_CODE_ERROR: i32 = 1;

/// Headless Example Defaults
/// Capacity of frame buffer
pub const HEADLESS_BUFFER_CAPACITY: usize = 10;
/// Number of frames to capture in example
pub const HEADLESS_CAPTURE_COUNT: usize = 50;
/// Delay between captures
pub const HEADLESS_CAPTURE_DELAY_MS: u64 = 100;
/// Timeout for entire capture session
pub const HEADLESS_TIMEOUT_SECS: u64 = 10;
/// Timeout for frame polling
pub const HEADLESS_POLL_TIMEOUT_MS: u64 = 1000;
/// Timeout for audio packet polling
pub const HEADLESS_AUDIO_POLL_TIMEOUT_MS: u64 = 100;
/// Timeout for stopping session
pub const HEADLESS_STOP_TIMEOUT_MS: u64 = 10_000;
/// Output filename for video frame
pub const HEADLESS_FRAME_FILENAME: &str = "captured_frame.raw";
/// Output filename for audio data
pub const HEADLESS_AUDIO_FILENAME: &str = "captured_audio.raw";

/// Demo App Defaults
/// Default width
pub const DEMO_DEFAULT_WIDTH: u32 = 640;
/// Default height
pub const DEMO_DEFAULT_HEIGHT: u32 = 480;
/// Default FPS
pub const DEMO_DEFAULT_FPS: f32 = 30.0;

/// Audio Encoding (Opus)
pub const OPUS_SAMPLE_RATE: u32 = 48000;
/// Opus Encoding - Frame Duration (20ms)
pub const OPUS_FRAME_DURATION_MS: u32 = 20;
/// Opus Encoding - Samples per Frame (960 for 20ms at 48kHz)
pub const OPUS_FRAME_SAMPLES: usize = 960; // 20ms at 48kHz
/// Opus Encoding - Audio Application Profile
pub const OPUS_APPLICATION_AUDIO: i32 = 2049;
/// Opus Encoding - `VoIP` Application Profile
pub const OPUS_APPLICATION_VOIP: i32 = 2048;
/// Opus Encoding - Low Delay Application Profile
pub const OPUS_APPLICATION_LOW_DELAY: i32 = 2051;

/// Blur Detection - Variance Thresholds
/// Threshold for extremely sharp images
pub const BLUR_VARIANCE_SHARP: f64 = 1000.0;
/// Threshold for good quality images
pub const BLUR_VARIANCE_GOOD: f64 = 500.0;
/// Threshold for moderately sharp images
pub const BLUR_VARIANCE_MODERATE: f64 = 200.0;
/// Threshold for blurry images
pub const BLUR_VARIANCE_BLURRY: f64 = 50.0;

/// Blur Detection - Quality Scores
/// Score for sharp images (1.0)
pub const QUALITY_SCORE_SHARP: f32 = 1.0;
/// Score for good images (0.8)
pub const QUALITY_SCORE_GOOD: f32 = 0.8;
/// Score for moderately sharp images (0.6)
pub const QUALITY_SCORE_MODERATE: f32 = 0.6;
/// Score for blurry images (0.3)
pub const QUALITY_SCORE_BLURRY: f32 = 0.3;
/// Score for very blurry images (0.1)
pub const QUALITY_SCORE_VERY_BLURRY: f32 = 0.1;

/// Blur Detection - Default Thresholds
/// Default variance threshold
pub const DEFAULT_VARIANCE_THRESHOLD: f64 = 200.0;
/// Default gradient threshold
pub const DEFAULT_GRADIENT_THRESHOLD: f64 = 50.0;

/// Image Processing - Luminance (Rec. 601)
/// Red channel weight
pub const LUMA_R: f32 = 0.299;
/// Green channel weight
pub const LUMA_G: f32 = 0.587;
/// Blue channel weight
pub const LUMA_B: f32 = 0.114;

/// Image Processing - Pyramids
/// Size of pooling window
pub const PYRAMID_POOLING_SIZE: usize = 2;
/// Area of pooling window
#[allow(clippy::cast_possible_truncation)]
// usize→u32: product is always 4, well within u32 range
pub const PYRAMID_POOLING_AREA: u32 = (PYRAMID_POOLING_SIZE * PYRAMID_POOLING_SIZE) as u32;

/// Image Processing - Alignment
/// Rotation threshold considered significant
pub const ALIGNMENT_SIGNIFICANT_ROTATION: f32 = 0.01;
/// Scale threshold considered significant
pub const ALIGNMENT_SIGNIFICANT_SCALE: f32 = 0.01;
/// Sampling step for alignment
pub const ALIGNMENT_SAMPLING_STEP: usize = 4;

/// Focus Stacking - Bracket Limits
/// Minimum number of brackets (2)
pub const FOCUS_STACK_MIN_BRACKETS: u32 = 2;
/// Maximum number of brackets (10)
pub const FOCUS_STACK_MAX_BRACKETS: u32 = 10;
/// Minimum number of shots per bracket (1)
pub const FOCUS_STACK_MIN_SHOTS: u32 = 1;
/// Maximum number of shots per bracket (10)
pub const FOCUS_STACK_MAX_SHOTS: u32 = 10;
/// Minimum number of focus steps (2)
pub const FOCUS_STACK_MIN_STEPS: u32 = 2;
/// Maximum number of focus steps (100)
pub const FOCUS_STACK_MAX_STEPS: u32 = 100;
/// Minimum sharpness threshold (0.0)
pub const FOCUS_STACK_MIN_SHARPNESS: f32 = 0.0;
/// Maximum sharpness threshold (1.0)
pub const FOCUS_STACK_MAX_SHARPNESS: f32 = 1.0;

/// Focus Stacking - Focus Distance
/// Minimum focus distance (macro)
pub const FOCUS_STACK_MIN_DIST: f32 = 0.0;
/// Maximum focus distance (infinity)
pub const FOCUS_STACK_MAX_DIST: f32 = 1.0;

/// Capture Settings
/// Default retry count for capture operations
pub const CAPTURE_RETRY_COUNT: u32 = 3;
/// Number of warmup frames to discard
pub const CAPTURE_WARMUP_FRAMES: u32 = 5;
/// Delay between warmup frames in ms
pub const CAPTURE_WARMUP_DELAY_MS: u64 = 30;
/// Warmup frames after reconnection
pub const CAPTURE_RECONNECT_WARMUP_FRAMES: u32 = 10;
/// Delay between reconnection warmup frames in ms
pub const CAPTURE_RECONNECT_WARMUP_DELAY_MS: u64 = 50;
/// Maximum number of frames in a sequence
pub const CAPTURE_SEQUENCE_MAX_COUNT: u32 = 20;
/// Maximum number of frames in a burst
pub const BURST_MAX_COUNT: u32 = 50;

/// Platform - Connection
/// Initial backoff delay for connection retry
pub const CONNECTION_BACKOFF_INITIAL_MS: u64 = 100;
/// Maximum backoff delay for connection retry
pub const CONNECTION_BACKOFF_MAX_MS: u64 = 2000;
/// Default number of connection retries
pub const CONNECTION_RETRY_DEFAULT: u32 = 3;
/// Interval for device monitor polling
pub const DEVICE_MONITOR_POLL_INTERVAL_MS: u64 = 2000;

/// Platform - Mock Camera
/// Simulated capture latency (16.7ms for 60fps)
pub const MOCK_CAPTURE_LATENCY_MS: f32 = 16.7; // 60 FPS
/// Simulated processing time
pub const MOCK_PROCESSING_TIME_MS: f32 = 5.0;
/// Simulated memory usage
pub const MOCK_MEMORY_USAGE_MB: f32 = 32.0;
/// Simulated FPS
pub const MOCK_FPS: f32 = 60.0;
/// Simulated quality score
pub const MOCK_QUALITY_SCORE: f32 = 0.95;
/// Simulated slow capture delay
pub const MOCK_SLOW_CAPTURE_DELAY_MS: u64 = 100;

/// Platform - Windows Metadata
/// MJPEG Header Signature
pub const MJPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];
/// Percentage of non-zero bytes required to consider a frame valid
pub const VALID_FRAME_NONZERO_PERCENT: f64 = 1.0;

/// Focus Stacking - Defaults
/// Default delay between focus steps in ms
pub const FOCUS_STACK_DEFAULT_DELAY_MS: u32 = 200;
/// Default sharpness threshold
pub const FOCUS_STACK_DEFAULT_SHARPNESS: f32 = 0.5;
/// Default number of blend levels
pub const FOCUS_STACK_DEFAULT_BLEND_LEVELS: u32 = 5;
/// Default bracket overlap factor
pub const FOCUS_STACK_BRACKET_OVERLAP: f32 = 1.2;

/// Exposure Analysis - Brightness Thresholds
/// Threshold for low brightness
pub const EXPOSURE_BRIGHTNESS_LOW: f32 = 0.2;
/// Threshold for dark images
pub const EXPOSURE_BRIGHTNESS_DARK: f32 = 0.35;
/// Threshold for good brightness
pub const EXPOSURE_BRIGHTNESS_GOOD: f32 = 0.65;
/// Threshold for high brightness
pub const EXPOSURE_BRIGHTNESS_HIGH: f32 = 0.8;

/// Exposure Analysis - Pixel Thresholds
/// Pixel value considered dark (0-255)
pub const EXPOSURE_PIXEL_DARK: u8 = 30;
/// Pixel value considered bright (0-255)
pub const EXPOSURE_PIXEL_BRIGHT: u8 = 225;

/// Smart Trigger Defaults
/// Minimum quality score to trigger
pub const TRIGGER_MIN_QUALITY: f32 = 0.75;
/// Stability duration required to trigger in ms
pub const TRIGGER_STABILITY_MS: u64 = 200;
/// Timeout for trigger in seconds
pub const TRIGGER_TIMEOUT_SECS: u64 = 5;
/// Consecutive frames required to trigger
pub const TRIGGER_CONSECUTIVE_FRAMES: usize = 3;
/// Size of trigger history buffer
pub const TRIGGER_HISTORY_SIZE: usize = 10;

/// Video bitrate (Standard HD quality)
pub const VIDEO_BITRATE_HD: u32 = 5_000_000;

/// Recording - Audio Channel Capacity
pub const RECORDING_AUDIO_CHANNEL_CAPACITY: usize = 256;

/// Recording - Audio Thread Sleep Duration (ms)
pub const RECORDING_AUDIO_SLEEP_MS: u64 = 1;

/// Defaults
/// Default camera ID
pub const DEFAULT_CAMERA_ID: &str = "0";

/// Recording Quality Presets
/// Low quality preset ("low")
pub const RECORDING_QUALITY_PRESET_LOW: &str = "low";
/// 720p quality preset ("720p")
pub const RECORDING_QUALITY_PRESET_720P: &str = "720p";
/// Medium quality preset ("medium")
pub const RECORDING_QUALITY_PRESET_MEDIUM: &str = "medium";
/// 1080p quality preset ("1080p")
pub const RECORDING_QUALITY_PRESET_1080P: &str = "1080p";
/// High quality preset ("high")
pub const RECORDING_QUALITY_PRESET_HIGH: &str = "high";
/// 4K quality preset ("4k")
pub const RECORDING_QUALITY_PRESET_4K: &str = "4k";
/// Recording session ID prefix
pub const RECORDING_SESSION_PREFIX: &str = "rec_";

/// Permissions
/// Permission request timeout
pub const PERMISSION_REQUEST_TIMEOUT_SECS: u64 = 60;
#[cfg(target_os = "macos")]
/// macOS AVMediaTypeVideo
pub const AV_MEDIA_TYPE_VIDEO: &str = "vide";

/// Recording - Frame Drop Log Interval
pub const RECORDING_DROP_LOG_INTERVAL: u64 = 10;

/// Recording - Frame Jitter Tolerance (0.0-1.0)
/// Allows frames to be up to 20% early
pub const RECORDING_JITTER_TOLERANCE: f64 = 0.8;

/// Video bitrate (High quality/4K)
pub const VIDEO_BITRATE_4K: u32 = 10_000_000;

/// Video bitrate (Low quality/720p)
pub const VIDEO_BITRATE_SD: u32 = 2_500_000;
