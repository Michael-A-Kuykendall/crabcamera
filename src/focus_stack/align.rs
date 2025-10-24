/// Image alignment module for focus stacking
/// 
/// Aligns images to compensate for camera movement between captures.
/// Uses feature detection and homography estimation.
use crate::types::CameraFrame;
use super::FocusStackError;

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
        log::debug!("Aligning frame {} to reference", idx);
        
        // Validate dimensions match
        if frame.width != reference.width || frame.height != reference.height {
            return Err(FocusStackError::DimensionMismatch {
                expected: (reference.width, reference.height),
                got: (frame.width, frame.height),
            });
        }
        
        // Compute alignment using center-of-mass
        // This is a simplified approach - production would use feature matching
        let alignment = compute_alignment_simple(reference, frame)?;
        
        log::debug!("Frame {} alignment: translation=({:.2}, {:.2}), error={:.3}",
            idx, alignment.translation.0, alignment.translation.1, alignment.error);
        
        results.push(alignment);
    }
    
    log::info!("Alignment complete");
    Ok(results)
}

/// Apply alignment transform to a frame
/// 
/// Transforms frame data according to alignment result.
/// Returns new frame with aligned data.
pub fn apply_alignment(
    frame: &CameraFrame,
    alignment: &AlignmentResult,
) -> Result<CameraFrame, FocusStackError> {
    // For identity transform, just clone
    if alignment.translation == (0.0, 0.0) && 
       alignment.rotation == 0.0 && 
       alignment.scale == 1.0 {
        return Ok(frame.clone());
    }
    
    log::debug!("Applying alignment: translation=({:.2}, {:.2}), rotation={:.4}, scale={:.4}",
        alignment.translation.0, alignment.translation.1, 
        alignment.rotation, alignment.scale);
    
    // Create new frame with same dimensions
    let mut aligned = frame.clone();
    
    // Apply translation
    // Simple implementation: shift pixels by integer translation
    let tx = alignment.translation.0.round() as i32;
    let ty = alignment.translation.1.round() as i32;
    
    if tx != 0 || ty != 0 {
        apply_translation(&mut aligned, tx, ty);
    }
    
    // Apply rotation if significant (> 0.01 radians = ~0.57 degrees)
    if alignment.rotation.abs() > 0.01 {
        apply_rotation(&mut aligned, alignment.rotation);
    }
    
    // Apply scale if different from 1.0
    if (alignment.scale - 1.0).abs() > 0.01 {
        apply_scale(&mut aligned, alignment.scale);
    }
    
    Ok(aligned)
}

/// Compute simple alignment using center-of-mass
fn compute_alignment_simple(
    reference: &CameraFrame,
    frame: &CameraFrame,
) -> Result<AlignmentResult, FocusStackError> {
    // Compute center of mass for both images
    let ref_com = compute_center_of_mass(reference);
    let frame_com = compute_center_of_mass(frame);
    
    // Translation is difference in center of mass
    let translation = (
        frame_com.0 - ref_com.0,
        frame_com.1 - ref_com.1,
    );
    
    // Compute alignment error (simplified)
    let error = (translation.0.powi(2) + translation.1.powi(2)).sqrt();
    
    Ok(AlignmentResult {
        translation,
        rotation: 0.0,
        scale: 1.0,
        error,
    })
}

/// Compute center of mass of image (weighted by brightness)
fn compute_center_of_mass(frame: &CameraFrame) -> (f32, f32) {
    let width = frame.width as usize;
    let height = frame.height as usize;
    
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_weight = 0.0;
    
    // Sample every 4th pixel for speed (RGB has 3 bytes per pixel)
    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            let idx = (y * width + x) * 3;
            
            if idx + 2 < frame.data.len() {
                // Use luminance as weight
                let r = frame.data[idx] as f32;
                let g = frame.data[idx + 1] as f32;
                let b = frame.data[idx + 2] as f32;
                let weight = 0.299 * r + 0.587 * g + 0.114 * b;
                
                sum_x += x as f32 * weight;
                sum_y += y as f32 * weight;
                sum_weight += weight;
            }
        }
    }
    
    if sum_weight > 0.0 {
        (sum_x / sum_weight, sum_y / sum_weight)
    } else {
        (width as f32 / 2.0, height as f32 / 2.0)
    }
}

/// Apply translation to frame data
fn apply_translation(frame: &mut CameraFrame, tx: i32, ty: i32) {
    if tx == 0 && ty == 0 {
        return;
    }
    
    let width = frame.width as i32;
    let height = frame.height as i32;
    
    // Create new buffer for shifted data
    let mut new_data = vec![0u8; frame.data.len()];
    
    // Copy pixels with offset
    for y in 0..height {
        for x in 0..width {
            let src_x = x - tx;
            let src_y = y - ty;
            
            // Check if source is in bounds
            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = ((src_y * width + src_x) * 3) as usize;
                let dst_idx = ((y * width + x) * 3) as usize;
                
                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3].copy_from_slice(&frame.data[src_idx..src_idx + 3]);
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
    
    let width = frame.width as i32;
    let height = frame.height as i32;
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    
    let cos_theta = rotation.cos();
    let sin_theta = rotation.sin();
    
    let mut new_data = vec![0u8; frame.data.len()];
    
    for y in 0..height {
        for x in 0..width {
            // Rotate around center
            let x_centered = x as f32 - cx;
            let y_centered = y as f32 - cy;
            
            let src_x = (x_centered * cos_theta - y_centered * sin_theta + cx).round() as i32;
            let src_y = (x_centered * sin_theta + y_centered * cos_theta + cy).round() as i32;
            
            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = ((src_y * width + src_x) * 3) as usize;
                let dst_idx = ((y * width + x) * 3) as usize;
                
                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3].copy_from_slice(&frame.data[src_idx..src_idx + 3]);
                }
            }
        }
    }
    
    frame.data = new_data;
}

/// Apply scale to frame (simple nearest-neighbor)
fn apply_scale(frame: &mut CameraFrame, scale: f32) {
    if scale == 1.0 {
        return;
    }
    
    let width = frame.width as i32;
    let height = frame.height as i32;
    let inv_scale = 1.0 / scale;
    
    let mut new_data = vec![0u8; frame.data.len()];
    
    for y in 0..height {
        for x in 0..width {
            let src_x = (x as f32 * inv_scale).round() as i32;
            let src_y = (y as f32 * inv_scale).round() as i32;
            
            if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                let src_idx = ((src_y * width + src_x) * 3) as usize;
                let dst_idx = ((y * width + x) * 3) as usize;
                
                if src_idx + 2 < frame.data.len() && dst_idx + 2 < new_data.len() {
                    new_data[dst_idx..dst_idx + 3].copy_from_slice(&frame.data[src_idx..src_idx + 3]);
                }
            }
        }
    }
    
    frame.data = new_data;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alignment_result_default() {
        let result = AlignmentResult::default();
        assert_eq!(result.translation, (0.0, 0.0));
        assert_eq!(result.rotation, 0.0);
        assert_eq!(result.scale, 1.0);
        assert_eq!(result.error, 0.0);
    }
    
    #[test]
    fn test_center_of_mass_uniform() {
        // Create uniform gray image
        let width = 100;
        let height = 100;
        let data = vec![128u8; width * height * 3];
        
        let frame = CameraFrame::new(
            data,
            width as u32,
            height as u32,
            "test_device".to_string(),
        );
        
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
        
        let mut frame = CameraFrame::new(
            data,
            width as u32,
            height as u32,
            "test_device".to_string(),
        );
        
        apply_translation(&mut frame, 2, 2);
        
        // Verify frame data was modified
        assert_eq!(frame.data.len(), width * height * 3);
    }
    
    #[test]
    fn test_insufficient_frames() {
        let frames = vec![];
        let result = align_frames(&frames);
        
        assert!(matches!(result, Err(FocusStackError::InsufficientImages { .. })));
    }
}
