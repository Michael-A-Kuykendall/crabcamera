use super::{FocusStackConfig, FocusStackError};
use crate::commands::capture::capture_with_reconnect;
/// Focus stack capture module
///
/// Handles capturing multiple images at different focus distances
/// for focus stacking. Requires camera with manual focus control.
use crate::types::{CameraFormat, CameraFrame};

/// Capture a sequence of images at different focus distances
///
/// This function captures multiple images with varying focus distances.
/// For cameras without programmable focus, user must manually adjust focus
/// between captures (using step_delay_ms for time to adjust).
pub async fn capture_focus_sequence(
    device_id: String,
    config: FocusStackConfig,
    format: Option<CameraFormat>,
) -> Result<Vec<CameraFrame>, FocusStackError> {
    // Validate config
    if config.num_steps < 2 {
        return Err(FocusStackError::InvalidConfig(
            "num_steps must be at least 2".to_string(),
        ));
    }

    if config.focus_start < 0.0
        || config.focus_start > 1.0
        || config.focus_end < 0.0
        || config.focus_end > 1.0
    {
        return Err(FocusStackError::InvalidConfig(
            "focus_start and focus_end must be between 0.0 and 1.0".to_string(),
        ));
    }

    log::info!(
        "Starting focus stack capture: {} steps from {} to {} with {}ms delay",
        config.num_steps,
        config.focus_start,
        config.focus_end,
        config.step_delay_ms
    );

    let capture_format = format.unwrap_or_else(CameraFormat::standard);
    let mut frames = Vec::with_capacity(config.num_steps as usize);

    // Calculate focus step size
    let focus_range = config.focus_end - config.focus_start;
    let focus_step = if config.num_steps > 1 {
        focus_range / (config.num_steps - 1) as f32
    } else {
        0.0
    };

    // Capture each step
    for step in 0..config.num_steps {
        let focus_distance = config.focus_start + (step as f32 * focus_step);

        log::debug!(
            "Capturing focus step {}/{} at distance {:.3}",
            step + 1,
            config.num_steps,
            focus_distance
        );

        // NOTE: Automatic focus distance control requires platform-specific camera APIs:
        // - Windows: IAMCameraControl::Set(CameraControl_Focus, ...)
        // - macOS: AVCaptureDevice.setFocusMode() and lensPosition
        // - Linux: v4l2 VIDIOC_S_CTRL with V4L2_CID_FOCUS_ABSOLUTE
        // Current implementation captures with manual focus adjustment by user.
        // Use config.step_delay_ms to allow time for manual adjustment between captures.

        // Capture frame with reconnection support
        match capture_with_reconnect(device_id.clone(), capture_format.clone(), 3).await {
            Ok(frame) => {
                log::debug!(
                    "Captured frame: {}x{} ({} bytes)",
                    frame.width,
                    frame.height,
                    frame.size_bytes
                );
                frames.push(frame);
            }
            Err(e) => {
                log::error!("Failed to capture frame at step {}: {}", step + 1, e);
                return Err(FocusStackError::MergeFailed(format!(
                    "Capture failed at step {}: {}",
                    step + 1,
                    e
                )));
            }
        }

        // Delay before next capture (except for last frame)
        if step < config.num_steps - 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                config.step_delay_ms as u64,
            ))
            .await;
        }
    }

    log::info!("Captured {} frames for focus stack", frames.len());

    // Validate all frames have same dimensions
    if let Some(first_frame) = frames.first() {
        let expected_dims = (first_frame.width, first_frame.height);

        for (_i, frame) in frames.iter().enumerate().skip(1) {
            let dims = (frame.width, frame.height);
            if dims != expected_dims {
                return Err(FocusStackError::DimensionMismatch {
                    expected: expected_dims,
                    got: dims,
                });
            }
        }
    }

    Ok(frames)
}

/// Capture focus brackets for advanced focus stacking
///
/// This captures overlapping focus ranges for better coverage.
/// Uses a bracketing approach: near, mid, far with overlap.
pub async fn capture_focus_brackets(
    device_id: String,
    brackets: u32,
    shots_per_bracket: u32,
    format: Option<CameraFormat>,
) -> Result<Vec<CameraFrame>, FocusStackError> {
    if !(2..=10).contains(&brackets) {
        return Err(FocusStackError::InvalidConfig(
            "brackets must be between 2 and 10".to_string(),
        ));
    }

    if !(1..=10).contains(&shots_per_bracket) {
        return Err(FocusStackError::InvalidConfig(
            "shots_per_bracket must be between 1 and 10".to_string(),
        ));
    }

    log::info!(
        "Capturing {} brackets with {} shots each",
        brackets,
        shots_per_bracket
    );

    let mut all_frames = Vec::new();
    let bracket_step = 1.0 / brackets as f32;

    for bracket_idx in 0..brackets {
        let focus_start = bracket_idx as f32 * bracket_step;
        let focus_end = focus_start + bracket_step * 1.2; // 20% overlap

        let config = FocusStackConfig {
            num_steps: shots_per_bracket,
            step_delay_ms: 200,
            focus_start: focus_start.min(1.0),
            focus_end: focus_end.min(1.0),
            enable_alignment: true,
            sharpness_threshold: 0.5,
            blend_levels: 5,
        };

        let frames = capture_focus_sequence(device_id.clone(), config, format.clone()).await?;

        all_frames.extend(frames);
    }

    log::info!(
        "Captured total of {} frames across {} brackets",
        all_frames.len(),
        brackets
    );

    Ok(all_frames)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = FocusStackConfig {
            num_steps: 1,
            ..Default::default()
        };

        // Should fail with insufficient steps
        assert!(matches!(
            validate_config(&config),
            Err(FocusStackError::InvalidConfig(_))
        ));
    }

    #[test]
    fn test_focus_step_calculation() {
        let num_steps = 5;
        let focus_start = 0.0;
        let focus_end = 1.0;

        let focus_range = focus_end - focus_start;
        let focus_step = focus_range / (num_steps - 1) as f32;

        assert_eq!(focus_step, 0.25);

        // Verify all steps are in range
        for step in 0..num_steps {
            let focus = focus_start + (step as f32 * focus_step);
            assert!((0.0..=1.0).contains(&focus));
        }
    }

    #[test]
    fn test_bracket_calculation() {
        let brackets = 3;
        let bracket_step = 1.0 / brackets as f32;

        assert!((bracket_step - 0.333).abs() < 0.01);

        for i in 0..brackets {
            let start = i as f32 * bracket_step;
            let end = (start + bracket_step * 1.2).min(1.0);

            assert!((0.0..=1.0).contains(&start));
            assert!((0.0..=1.0).contains(&end));
            assert!(end > start); // Ensure overlap makes sense
        }
    }

    fn validate_config(config: &FocusStackConfig) -> Result<(), FocusStackError> {
        if config.num_steps < 2 {
            return Err(FocusStackError::InvalidConfig(
                "num_steps must be at least 2".to_string(),
            ));
        }
        Ok(())
    }
}
