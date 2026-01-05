pub mod align;
/// Focus Stacking Module
///
/// Implements automated focus stacking for macro photography:
/// 1. Capture multiple images at different focus distances
/// 2. Align images to compensate for camera movement
/// 3. Detect sharp regions using edge detection
/// 4. Blend images using pyramid blending for smooth transitions
///
/// This is useful for macro photography where depth of field is limited.
pub mod capture;
pub mod merge;

use crate::types::CameraFrame;

/// Focus stack configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FocusStackConfig {
    /// Number of focus steps to capture
    pub num_steps: u32,

    /// Delay between captures (ms) - allows camera to stabilize
    pub step_delay_ms: u32,

    /// Focus distance start (0.0 = near, 1.0 = far)
    pub focus_start: f32,

    /// Focus distance end
    pub focus_end: f32,

    /// Enable alignment compensation
    pub enable_alignment: bool,

    /// Sharpness threshold for region detection (0.0-1.0)
    pub sharpness_threshold: f32,

    /// Pyramid blending levels (3-7 recommended)
    pub blend_levels: u32,
}

impl Default for FocusStackConfig {
    fn default() -> Self {
        Self {
            num_steps: 10,
            step_delay_ms: 200,
            focus_start: 0.0,
            focus_end: 1.0,
            enable_alignment: true,
            sharpness_threshold: 0.5,
            blend_levels: 5,
        }
    }
}

/// Focus stack result containing the merged image and metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FocusStackResult {
    /// The final merged frame
    pub merged_frame: CameraFrame,

    /// Number of source images used
    pub num_sources: usize,

    /// Average alignment error (pixels)
    pub alignment_error: f32,

    /// Processing time (ms)
    pub processing_time_ms: u64,
}

/// Focus stack error types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FocusStackError {
    /// Not enough source images
    InsufficientImages { required: usize, provided: usize },

    /// Image dimensions don't match
    DimensionMismatch {
        expected: (u32, u32),
        got: (u32, u32),
    },

    /// Frame data is corrupted or wrong size
    DataCorruption {
        frame_size: usize,
        expected_size: usize,
    },

    /// Alignment failed
    AlignmentFailed(String),

    /// Merge failed
    MergeFailed(String),

    /// Invalid configuration
    InvalidConfig(String),
}

impl std::fmt::Display for FocusStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientImages { required, provided } => {
                write!(
                    f,
                    "Insufficient images: need {}, got {}",
                    required, provided
                )
            }
            Self::DimensionMismatch { expected, got } => {
                write!(
                    f,
                    "Image dimension mismatch: expected {}x{}, got {}x{}",
                    expected.0, expected.1, got.0, got.1
                )
            }
            Self::DataCorruption {
                frame_size,
                expected_size,
            } => {
                write!(
                    f,
                    "Frame data corruption: got {} bytes, expected {}",
                    frame_size, expected_size
                )
            }
            Self::AlignmentFailed(msg) => write!(f, "Alignment failed: {}", msg),
            Self::MergeFailed(msg) => write!(f, "Merge failed: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for FocusStackError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FocusStackConfig::default();
        assert_eq!(config.num_steps, 10);
        assert_eq!(config.step_delay_ms, 200);
        assert_eq!(config.focus_start, 0.0);
        assert_eq!(config.focus_end, 1.0);
        assert!(config.enable_alignment);
        assert_eq!(config.sharpness_threshold, 0.5);
        assert_eq!(config.blend_levels, 5);
    }

    #[test]
    fn test_config_validation() {
        let config = FocusStackConfig {
            num_steps: 2,
            ..Default::default()
        };

        // Test num_steps bounds
        assert!(config.num_steps >= 2);

        // Test focus range
        assert!(config.focus_start >= 0.0 && config.focus_start <= 1.0);
        assert!(config.focus_end >= 0.0 && config.focus_end <= 1.0);

        // Test threshold bounds
        assert!(config.sharpness_threshold >= 0.0 && config.sharpness_threshold <= 1.0);

        // Test blend levels reasonable
        assert!(config.blend_levels >= 3 && config.blend_levels <= 10);
    }

    #[test]
    fn test_error_display() {
        let err = FocusStackError::InsufficientImages {
            required: 5,
            provided: 2,
        };
        assert!(err.to_string().contains("Insufficient images"));

        let err = FocusStackError::DimensionMismatch {
            expected: (1920, 1080),
            got: (1280, 720),
        };
        assert!(err.to_string().contains("dimension mismatch"));
    }
}
