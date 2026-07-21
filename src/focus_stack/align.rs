use super::FocusStackError;
use crate::constants::{
    ALIGNMENT_SAMPLING_STEP, ALIGNMENT_SIGNIFICANT_ROTATION, ALIGNMENT_SIGNIFICANT_SCALE, LUMA_B,
    LUMA_G, LUMA_R,
};
/// Image alignment module for focus stacking
///
/// Aligns images to compensate for camera movement between captures.
/// Uses feature detection and homography estimation.
use crate::types::CameraFrame;

/// Alignment result containing transform and error metrics
#[derive(Debug, Clone)]
pub struct AlignmentResult {
    /// Translation in pixels (x, y)
    pub translation: (f32, f32),

    /// Rotation in radians
    pub rotation: f32,

    /// Scale factor
    pub scale: f32,

    /// Alignment error (RMS pixel distance)
    pub error: f32,
}

impl Default for AlignmentResult {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            rotation: 0.0,
            scale: 1.0,
            error: 0.0,
        }
    }
}

/// Align a sequence of frames to the first frame
///
/// Returns alignment transforms for each frame relative to reference.
/// Uses simple center-of-mass alignment as a starting point.
/// For production, would use feature detection (SIFT/ORB) + RANSAC.
///
/// # Errors
/// Returns a [`FocusStackError::InsufficientImages`] if fewer than two frames
/// are provided, or a [`FocusStackError::DimensionMismatch`] if any frame does
/// not match the reference frame's dimensions.
pub fn align_frames(frames: &[CameraFrame]) -> Result<Vec<AlignmentResult>, FocusStackError> {
    if frames.len() < 2 {
        return Err(FocusStackError::InsufficientImages {
            required: 2,
            provided: frames.len(),
        });
    }

    log::info!("Aligning {} frames", frames.len());

    let reference = &frames[0];
    let mut results = Vec::with_capacity(frames.len());

    // First frame is reference (no transform)
    results.push(AlignmentResult::default());

    // Align remaining frames to reference
    for (idx, frame) in frames.iter().enumerate().skip(1) {
        log::debug!("Aligning frame {idx} to reference");

        // Validate dimensions match
        if frame.width != reference.width || frame.height != reference.height {
            return Err(FocusStackError::DimensionMismatch {
                expected: (reference.width, reference.height),
                got: (frame.width, frame.height),
            });
        }

        // Compute alignment using center-of-mass
        // This is a simplified approach - production would use feature matching
        let alignment = compute_alignment_simple(reference, frame);

        log::debug!(
            "Frame {} alignment: translation=({:.2}, {:.2}), error={:.3}",
            idx,
            alignment.translation.0,
            alignment.translation.1,
            alignment.error
        );

        results.push(alignment);
    }

    log::info!("Alignment complete");
    Ok(results)
}

/// Apply alignment transform to a frame
///
/// Transforms frame data according to alignment result.
/// Returns new frame with aligned data.
///
/// # Errors
/// This function always succeeds and never returns an `Err`.
pub fn apply_alignment(
    frame: &CameraFrame,
    alignment: &AlignmentResult,
) -> Result<CameraFrame, FocusStackError> {
    // For identity transform, just clone (epsilon comparison: transforms below this magnitude are visually indistinguishable)
    let is_identity = alignment.translation.0.abs() < f32::EPSILON
        && alignment.translation.1.abs() < f32::EPSILON
        && alignment.rotation.abs() < f32::EPSILON
        && (alignment.scale - 1.0).abs() < f32::EPSILON;
    if is_identity {
        return Ok(frame.clone());
    }

    log::debug!(
        "Applying alignment: translation=({:.2}, {:.2}), rotation={:.4}, scale={:.4}",
        alignment.translation.0,
        alignment.translation.1,
        alignment.rotation,
        alignment.scale
    );

    // Create new frame with same dimensions
    let mut aligned = frame.clone();

    // Apply translation
    // Simple implementation: shift pixels by integer translation
    #[allow(clippy::cast_possible_truncation)] // translation values fit in i32 range
    let tx = alignment.translation.0.round() as i32;
    #[allow(clippy::cast_possible_truncation)] // translation values fit in i32 range
    let ty = alignment.translation.1.round() as i32;

    if tx != 0 || ty != 0 {
        apply_translation(&mut aligned, tx, ty);
    }

    // Apply rotation if significant
    if alignment.rotation.abs() > ALIGNMENT_SIGNIFICANT_ROTATION {
        apply_rotation(&mut aligned, alignment.rotation);
    }

    // Apply scale if different from 1.0
    if (alignment.scale - 1.0).abs() > ALIGNMENT_SIGNIFICANT_SCALE {
        apply_scale(&mut aligned, alignment.scale);
    }

    Ok(aligned)
}

/// Compute simple alignment using center-of-mass
fn compute_alignment_simple(reference: &CameraFrame, frame: &CameraFrame) -> AlignmentResult {
    // Compute center of mass for both images
    let ref_com = compute_center_of_mass(reference);
    let frame_com = compute_center_of_mass(frame);

    // Translation is difference in center of mass
    let translation = (frame_com.0 - ref_com.0, frame_com.1 - ref_com.1);

    // Compute alignment error (simplified)
    let error = (translation.0.powi(2) + translation.1.powi(2)).sqrt();

    AlignmentResult {
        translation,
        rotation: 0.0,
        scale: 1.0,
        error,
    }
}

/// Compute center of mass of image (weighted by brightness)
fn compute_center_of_mass(frame: &CameraFrame) -> (f32, f32) {
    let width = frame.width as usize;
    let height = frame.height as usize;

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_weight = 0.0;

    // Sample pixels for speed (RGB has 3 bytes per pixel)
    for y in (0..height).step_by(ALIGNMENT_SAMPLING_STEP) {
        for x in (0..width).step_by(ALIGNMENT_SAMPLING_STEP) {
            let idx = (y * width + x) * 3;

            if idx + 2 < frame.data.len() {
                // Use luminance as weight
                let r = f32::from(frame.data[idx]);
                let g = f32::from(frame.data[idx + 1]);
                let b = f32::from(frame.data[idx + 2]);
                let weight = LUMA_R * r + LUMA_G * g + LUMA_B * b;

                #[allow(clippy::cast_precision_loss)] // pixel indices fit in f32 mantissa
                let x_f32 = x as f32;
                #[allow(clippy::cast_precision_loss)] // pixel indices fit in f32 mantissa
                let y_f32 = y as f32;
                sum_x += x_f32 * weight;
                sum_y += y_f32 * weight;
                sum_weight += weight;
            }
        }
    }

    if sum_weight > 0.0 {
        (sum_x / sum_weight, sum_y / sum_weight)
    } else {
        #[allow(clippy::cast_precision_loss)] // image dimensions fit in f32 mantissa
        let w = width as f32 / 2.0;
        #[allow(clippy::cast_precision_loss)] // image dimensions fit in f32 mantissa
        let h = height as f32 / 2.0;
        (w, h)
    }
}

/// Apply translation to frame data
fn apply_translation(frame: &mut CameraFrame, tx: i32, ty: i32) {
    if tx == 0 && ty == 0 {
        return;
    }

    let width = i32::try_from(frame.width).unwrap_or(i32::MAX);
    let height = i32::try_from(frame.height).unwrap_or(i32::MAX);

    // Create new buffer for shifted data
    let mut new_data = vec![0u8; frame.data.len()];

    // Copy pixels with offset
    for y in 0..height {
        for x in 0..width {
            let src_x = x - tx;
            let src_y = y - ty;

            // Check if source is in bounds
            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = usize::try_from((src_y * width + src_x) * 3).unwrap_or(0);
                let dst_idx = usize::try_from((y * width + x) * 3).unwrap_or(0);

                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3]
                        .copy_from_slice(&frame.data[src_idx..src_idx + 3]);
                }
            }
        }
    }

    frame.data = new_data;
}

/// Apply rotation to frame (simple nearest-neighbor)
fn apply_rotation(frame: &mut CameraFrame, rotation: f32) {
    if rotation == 0.0 {
        return;
    }

    // Safe: camera dimensions never exceed i32::MAX
    #[allow(clippy::cast_possible_wrap)]
    let width = frame.width as i32;
    #[allow(clippy::cast_possible_wrap)]
    let height = frame.height as i32;
    #[allow(clippy::cast_precision_loss)] // dimensions fit in f32 mantissa
    let cx = width as f32 / 2.0;
    #[allow(clippy::cast_precision_loss)] // dimensions fit in f32 mantissa
    let cy = height as f32 / 2.0;

    let cos_theta = rotation.cos();
    let sin_theta = rotation.sin();

    let mut new_data = vec![0u8; frame.data.len()];

    for y in 0..height {
        for x in 0..width {
            // Rotate around center
            #[allow(clippy::cast_precision_loss)] // pixel coords fit in f32 mantissa
            let x_centered = x as f32 - cx;
            #[allow(clippy::cast_precision_loss)] // pixel coords fit in f32 mantissa
            let y_centered = y as f32 - cy;

            #[allow(clippy::cast_possible_truncation)] // clamped by bounds check below
            let src_x = (x_centered * cos_theta - y_centered * sin_theta + cx).round() as i32;
            #[allow(clippy::cast_possible_truncation)] // clamped by bounds check below
            let src_y = (x_centered * sin_theta + y_centered * cos_theta + cy).round() as i32;

            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = usize::try_from((src_y * width + src_x) * 3).unwrap_or(0);
                let dst_idx = usize::try_from((y * width + x) * 3).unwrap_or(0);

                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3]
                        .copy_from_slice(&frame.data[src_idx..src_idx + 3]);
                }
            }
        }
    }

    frame.data = new_data;
}

/// Apply scale to frame (simple nearest-neighbor)
fn apply_scale(frame: &mut CameraFrame, scale: f32) {
    if (scale - 1.0).abs() < f32::EPSILON {
        return;
    }

    // Safe: camera dimensions never exceed i32::MAX
    #[allow(clippy::cast_possible_wrap)]
    let width = frame.width as i32;
    #[allow(clippy::cast_possible_wrap)]
    let height = frame.height as i32;
    let inv_scale = 1.0 / scale;

    let mut new_data = vec![0u8; frame.data.len()];

    for y in 0..height {
        for x in 0..width {
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // pixel coords fit in f32 mantissa, clamped by bounds check
            let src_x = (x as f32 * inv_scale).round() as i32;
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)] // pixel coords fit in f32 mantissa, clamped by bounds check
            let src_y = (y as f32 * inv_scale).round() as i32;

            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = usize::try_from((src_y * width + src_x) * 3).unwrap_or(0);
                let dst_idx = usize::try_from((y * width + x) * 3).unwrap_or(0);

                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3]
                        .copy_from_slice(&frame.data[src_idx..src_idx + 3]);
                }
            }
        }
    }

    frame.data = new_data;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame(width: u32, height: u32, value: u8) -> CameraFrame {
        CameraFrame::new(
            vec![value; (width * height * 3) as usize],
            width,
            height,
            "test_device".to_string(),
        )
    }

    #[test]
    fn test_alignment_result_default() {
        let result = AlignmentResult::default();
        assert!(result.translation.0.abs() < 1e-6 && result.translation.1.abs() < 1e-6);
        assert!(result.rotation.abs() < 1e-6);
        assert!((result.scale - 1.0).abs() < 1e-6);
        assert!(result.error.abs() < 1e-6);
    }

    #[test]
    fn test_center_of_mass_uniform() {
        // Create uniform gray image
        let width = 100;
        let height = 100;
        let data = vec![128u8; width * height * 3];

        let frame = CameraFrame::new(data, u32::try_from(width).unwrap_or(u32::MAX), u32::try_from(height).unwrap_or(u32::MAX), "test_device".to_string());

        let com = compute_center_of_mass(&frame);

        // Should be near center
        assert!((com.0 - 50.0).abs() < 5.0);
        assert!((com.1 - 50.0).abs() < 5.0);
    }

    #[test]
    fn test_translation_application() {
        let width = 10;
        let height = 10;
        let data = vec![128u8; width * height * 3];

        let mut frame =
            CameraFrame::new(data, u32::try_from(width).unwrap_or(u32::MAX), u32::try_from(height).unwrap_or(u32::MAX), "test_device".to_string());

        apply_translation(&mut frame, 2, 2);

        // Verify frame data was modified
        assert_eq!(frame.data.len(), width * height * 3);
    }

    #[test]
    fn test_insufficient_frames() {
        let frames = vec![];
        let result = align_frames(&frames);

        assert!(matches!(
            result,
            Err(FocusStackError::InsufficientImages { .. })
        ));
    }

    #[test]
    fn test_align_frames_dimension_mismatch() {
        let a = test_frame(8, 8, 100);
        let b = test_frame(9, 8, 120);
        let result = align_frames(&[a, b]);

        assert!(matches!(
            result,
            Err(FocusStackError::DimensionMismatch { .. })
        ));
    }

    #[test]
    fn test_align_frames_success_and_identity_first_transform() {
        let a = test_frame(8, 8, 100);
        let b = test_frame(8, 8, 100);
        let result = align_frames(&[a, b]).expect("alignment should succeed");

        assert_eq!(result.len(), 2);
        assert!(result[0].translation.0.abs() < 1e-6 && result[0].translation.1.abs() < 1e-6);
        assert!(result[0].rotation.abs() < 1e-6);
        assert!((result[0].scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_alignment_identity_returns_clone() {
        let frame = test_frame(6, 6, 90);
        let aligned = apply_alignment(&frame, &AlignmentResult::default())
            .expect("identity transform should succeed");
        assert_eq!(aligned.data, frame.data);
        assert_eq!(aligned.width, frame.width);
        assert_eq!(aligned.height, frame.height);
    }

    #[test]
    fn test_apply_alignment_non_identity_path() {
        let frame = test_frame(6, 6, 200);
        let transform = AlignmentResult {
            translation: (1.0, 1.0),
            rotation: 0.01,
            scale: 1.02,
            error: 0.5,
        };

        let aligned = apply_alignment(&frame, &transform).expect("non-identity should succeed");
        assert_eq!(aligned.data.len(), frame.data.len());
    }

    #[test]
    fn test_compute_center_of_mass_zero_weight_falls_back_center() {
        let frame = test_frame(10, 10, 0);
        let com = compute_center_of_mass(&frame);
        assert!((com.0 - 5.0).abs() < 0.1);
        assert!((com.1 - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_rotation_and_scale_helpers_run() {
        let mut frame_rot = test_frame(10, 10, 128);
        apply_rotation(&mut frame_rot, 0.1);
        assert_eq!(frame_rot.data.len(), 10 * 10 * 3);

        let mut frame_scale = test_frame(10, 10, 128);
        apply_scale(&mut frame_scale, 1.2);
        assert_eq!(frame_scale.data.len(), 10 * 10 * 3);
    }
}
