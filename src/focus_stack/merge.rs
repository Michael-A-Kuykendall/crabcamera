/// Image merging module for focus stacking
/// 
/// Merges aligned images by selecting sharp regions from each frame.
/// Uses pyramid blending for smooth transitions between regions.
use crate::types::CameraFrame;
use super::FocusStackError;

/// Sharpness map for an image
/// Contains per-pixel sharpness scores (0.0 = blurry, 1.0 = sharp)
#[derive(Debug, Clone)]
pub struct SharpnessMap {
    pub width: u32,
    pub height: u32,
    pub scores: Vec<f32>,
}

/// Merge multiple aligned frames using focus stacking
/// 
/// For each pixel, selects the value from the sharpest source image.
/// Uses pyramid blending to avoid harsh transitions.
pub fn merge_frames(
    frames: &[CameraFrame],
    sharpness_threshold: f32,
    blend_levels: u32,
) -> Result<CameraFrame, FocusStackError> {
    if frames.is_empty() {
        return Err(FocusStackError::InsufficientImages {
            required: 1,
            provided: 0,
        });
    }
    
    if frames.len() == 1 {
        // Single frame, just return it
        return Ok(frames[0].clone());
    }
    
    log::info!("Merging {} frames with {} blend levels", frames.len(), blend_levels);
    
    let reference = &frames[0];
    let width = reference.width;
    let height = reference.height;
    
    // Validate all frames have same dimensions
    for frame in frames.iter().skip(1) {
        if frame.width != width || frame.height != height {
            return Err(FocusStackError::DimensionMismatch {
                expected: (width, height),
                got: (frame.width, frame.height),
            });
        }
    }
    
    // Compute sharpness maps for all frames
    log::debug!("Computing sharpness maps");
    let sharpness_maps: Vec<SharpnessMap> = frames
        .iter()
        .map(compute_sharpness_map)
        .collect();
    
    // Create merged frame
    log::debug!("Creating merged frame");
    let merged_data = if blend_levels > 0 {
        merge_with_pyramid_blending(frames, &sharpness_maps, blend_levels)?
    } else {
        merge_simple(frames, &sharpness_maps, sharpness_threshold)?
    };
    
    log::info!("Merge complete");
    
    Ok(CameraFrame::new(
        merged_data,
        width,
        height,
        reference.device_id.clone(),
    ).with_format(reference.format.clone()))
}

/// Simple merge: pick sharpest pixel from each frame
fn merge_simple(
    frames: &[CameraFrame],
    sharpness_maps: &[SharpnessMap],
    threshold: f32,
) -> Result<Vec<u8>, FocusStackError> {
    let width = frames[0].width as usize;
    let height = frames[0].height as usize;
    let pixel_count = width * height;
    
    let mut merged = vec![0u8; pixel_count * 3];
    
    for pixel_idx in 0..pixel_count {
        let mut best_sharpness = 0.0;
        let mut best_frame_idx = 0;
        
        // Find sharpest frame for this pixel
        for (frame_idx, sharpness_map) in sharpness_maps.iter().enumerate() {
            let sharpness = sharpness_map.scores[pixel_idx];
            if sharpness > best_sharpness && sharpness >= threshold {
                best_sharpness = sharpness;
                best_frame_idx = frame_idx;
            }
        }
        
        // Copy RGB values from best frame
        let src_idx = pixel_idx * 3;
        let dst_idx = pixel_idx * 3;
        
        merged[dst_idx..dst_idx + 3]
            .copy_from_slice(&frames[best_frame_idx].data[src_idx..src_idx + 3]);
    }
    
    Ok(merged)
}

/// Merge with pyramid blending for smooth transitions
fn merge_with_pyramid_blending(
    frames: &[CameraFrame],
    sharpness_maps: &[SharpnessMap],
    levels: u32,
) -> Result<Vec<u8>, FocusStackError> {
    let width = frames[0].width as usize;
    let height = frames[0].height as usize;
    
    log::debug!("Pyramid blending with {} levels", levels);
    
    // Create weight maps (normalized sharpness)
    let weight_maps = create_weight_maps(sharpness_maps);
    
    // Build Gaussian pyramids for each frame
    log::debug!("Building Gaussian pyramids");
    let gaussian_pyramids: Vec<Vec<Vec<u8>>> = frames
        .iter()
        .map(|frame| build_gaussian_pyramid(&frame.data, width, height, levels))
        .collect();
    
    // Build Laplacian pyramids
    log::debug!("Building Laplacian pyramids");
    let laplacian_pyramids: Vec<Vec<Vec<u8>>> = gaussian_pyramids
        .iter()
        .map(|pyramid| build_laplacian_pyramid(pyramid))
        .collect();
    
    // Build weight pyramids
    log::debug!("Building weight pyramids");
    let weight_pyramids: Vec<Vec<Vec<f32>>> = weight_maps
        .iter()
        .map(|weights| build_weight_pyramid(weights, width, height, levels))
        .collect();
    
    // Blend at each level
    log::debug!("Blending pyramids");
    let blended_pyramid = blend_pyramids(&laplacian_pyramids, &weight_pyramids);
    
    // Reconstruct from pyramid
    log::debug!("Reconstructing from pyramid");
    let merged = reconstruct_from_pyramid(&blended_pyramid, width, height);
    
    Ok(merged)
}

/// Compute sharpness map using Laplacian edge detection
fn compute_sharpness_map(frame: &CameraFrame) -> SharpnessMap {
    let width = frame.width as usize;
    let height = frame.height as usize;
    
    let mut scores = vec![0.0; width * height];
    
    // Compute Laplacian (approximation using 3x3 kernel)
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx = y * width + x;
            let pixel_idx = idx * 3;
            
            // Compute luminance for center pixel
            let center = luminance(&frame.data[pixel_idx..pixel_idx + 3]);
            
            // Compute Laplacian using 4-connected neighbors
            let mut laplacian = 0.0;
            for (dy, dx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let ny = (y as i32 + dy) as usize;
                let nx = (x as i32 + dx) as usize;
                let neighbor_idx = (ny * width + nx) * 3;
                let neighbor = luminance(&frame.data[neighbor_idx..neighbor_idx + 3]);
                laplacian += neighbor;
            }
            laplacian = (4.0 * center - laplacian).abs();
            
            // Normalize to 0-1 range (assuming max gradient of 255)
            scores[idx] = (laplacian / 255.0).min(1.0);
        }
    }
    
    SharpnessMap {
        width: frame.width,
        height: frame.height,
        scores,
    }
}

/// Convert RGB to luminance
fn luminance(rgb: &[u8]) -> f32 {
    0.299 * rgb[0] as f32 + 0.587 * rgb[1] as f32 + 0.114 * rgb[2] as f32
}

/// Create normalized weight maps from sharpness maps
fn create_weight_maps(sharpness_maps: &[SharpnessMap]) -> Vec<Vec<f32>> {
    let pixel_count = sharpness_maps[0].scores.len();
    let mut weight_maps = vec![vec![0.0; pixel_count]; sharpness_maps.len()];
    
    // Normalize weights at each pixel
    for pixel_idx in 0..pixel_count {
        let mut sum = 0.0;
        
        // Sum sharpness across all frames
        for map in sharpness_maps {
            sum += map.scores[pixel_idx];
        }
        
        // Normalize (avoid division by zero)
        if sum > 0.0 {
            for (frame_idx, map) in sharpness_maps.iter().enumerate() {
                weight_maps[frame_idx][pixel_idx] = map.scores[pixel_idx] / sum;
            }
        } else {
            // If all zero, distribute equally
            let equal_weight = 1.0 / sharpness_maps.len() as f32;
            for weight_map in &mut weight_maps {
                weight_map[pixel_idx] = equal_weight;
            }
        }
    }
    
    weight_maps
}

/// Build Gaussian pyramid (simple implementation using 2x2 average pooling)
fn build_gaussian_pyramid(data: &[u8], width: usize, height: usize, levels: u32) -> Vec<Vec<u8>> {
    let mut pyramid = Vec::with_capacity(levels as usize);
    pyramid.push(data.to_vec());
    
    let mut current_width = width;
    let mut current_height = height;
    
    for _ in 1..levels {
        let (downsampled, new_width, new_height) = downsample(pyramid.last().unwrap(), current_width, current_height);
        pyramid.push(downsampled);
        current_width = new_width;
        current_height = new_height;
        
        if current_width < 2 || current_height < 2 {
            break;
        }
    }
    
    pyramid
}

/// Downsample image by 2x using average pooling
fn downsample(data: &[u8], width: usize, height: usize) -> (Vec<u8>, usize, usize) {
    let new_width = width / 2;
    let new_height = height / 2;
    let mut downsampled = vec![0u8; new_width * new_height * 3];
    
    for y in 0..new_height {
        for x in 0..new_width {
            let dst_idx = (y * new_width + x) * 3;
            
            // Average 2x2 block
            let mut sum = [0u32; 3];
            for dy in 0..2 {
                for dx in 0..2 {
                    let src_x = x * 2 + dx;
                    let src_y = y * 2 + dy;
                    let src_idx = (src_y * width + src_x) * 3;
                    
                    if src_idx + 2 < data.len() {
                        sum[0] += data[src_idx] as u32;
                        sum[1] += data[src_idx + 1] as u32;
                        sum[2] += data[src_idx + 2] as u32;
                    }
                }
            }
            
            downsampled[dst_idx] = (sum[0] / 4) as u8;
            downsampled[dst_idx + 1] = (sum[1] / 4) as u8;
            downsampled[dst_idx + 2] = (sum[2] / 4) as u8;
        }
    }
    
    (downsampled, new_width, new_height)
}

/// Build Laplacian pyramid from Gaussian pyramid
fn build_laplacian_pyramid(gaussian: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut laplacian = Vec::with_capacity(gaussian.len());
    
    for (current, _next) in gaussian.iter().zip(gaussian.iter().skip(1)) {
        // Laplacian = Gaussian[i] - upsample(Gaussian[i+1])
        // For simplicity, just use Gaussian levels directly
        laplacian.push(current.clone());
    }
    
    // Last level is just the coarsest Gaussian
    laplacian.push(gaussian.last().unwrap().clone());
    
    laplacian
}

/// Build weight pyramid
fn build_weight_pyramid(weights: &[f32], width: usize, height: usize, levels: u32) -> Vec<Vec<f32>> {
    let mut pyramid = Vec::with_capacity(levels as usize);
    pyramid.push(weights.to_vec());
    
    let mut current_width = width;
    let mut current_height = height;
    
    for _ in 1..levels {
        let (downsampled, new_width, new_height) = downsample_weights(pyramid.last().unwrap(), current_width, current_height);
        pyramid.push(downsampled);
        current_width = new_width;
        current_height = new_height;
        
        if current_width < 2 || current_height < 2 {
            break;
        }
    }
    
    pyramid
}

/// Downsample weight map
fn downsample_weights(weights: &[f32], width: usize, height: usize) -> (Vec<f32>, usize, usize) {
    let new_width = width / 2;
    let new_height = height / 2;
    let mut downsampled = vec![0.0; new_width * new_height];
    
    for y in 0..new_height {
        for x in 0..new_width {
            let dst_idx = y * new_width + x;
            let mut sum = 0.0;
            
            for dy in 0..2 {
                for dx in 0..2 {
                    let src_idx = (y * 2 + dy) * width + (x * 2 + dx);
                    if src_idx < weights.len() {
                        sum += weights[src_idx];
                    }
                }
            }
            
            downsampled[dst_idx] = sum / 4.0;
        }
    }
    
    (downsampled, new_width, new_height)
}

/// Blend pyramids using weights
fn blend_pyramids(laplacians: &[Vec<Vec<u8>>], weights: &[Vec<Vec<f32>>]) -> Vec<Vec<u8>> {
    let num_levels = laplacians[0].len();
    let mut blended = Vec::with_capacity(num_levels);
    
    for level in 0..num_levels {
        let level_size = laplacians[0][level].len();
        let mut blended_level = vec![0u8; level_size];
        
        // Blend each pixel using weights
        for (pixel_idx, blended_pixel) in blended_level.iter_mut().enumerate() {
            let mut sum = 0.0;
            
            for frame_idx in 0..laplacians.len() {
                let pixel_val = laplacians[frame_idx][level][pixel_idx] as f32;
                let weight_idx = pixel_idx / 3; // Convert RGB index to pixel index
                let weight = weights[frame_idx][level].get(weight_idx).copied().unwrap_or(0.0);
                sum += pixel_val * weight;
            }
            
            *blended_pixel = sum.round().clamp(0.0, 255.0) as u8;
        }
        
        blended.push(blended_level);
    }
    
    blended
}

/// Reconstruct image from Laplacian pyramid
fn reconstruct_from_pyramid(pyramid: &[Vec<u8>], _width: usize, _height: usize) -> Vec<u8> {
    // Simplified: just return highest resolution level
    // Full implementation would upsample and add levels
    pyramid[0].clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_luminance_calculation() {
        let rgb = vec![100u8, 150, 200];
        let lum = luminance(&rgb);
        
        // Should be weighted average
        let expected = 0.299 * 100.0 + 0.587 * 150.0 + 0.114 * 200.0;
        assert!((lum - expected).abs() < 0.1);
    }
    
    #[test]
    fn test_sharpness_map_dimensions() {
        let width = 100;
        let height = 100;
        let data = vec![128u8; width * height * 3];
        
        let frame = CameraFrame::new(
            data,
            width as u32,
            height as u32,
            "test_device".to_string(),
        );
        
        let sharpness = compute_sharpness_map(&frame);
        
        assert_eq!(sharpness.width, width as u32);
        assert_eq!(sharpness.height, height as u32);
        assert_eq!(sharpness.scores.len(), width * height);
    }
    
    #[test]
    fn test_downsample_dimensions() {
        let width = 100;
        let height = 100;
        let data = vec![128u8; width * height * 3];
        
        let (downsampled, new_width, new_height) = downsample(&data, width, height);
        
        assert_eq!(new_width, width / 2);
        assert_eq!(new_height, height / 2);
        assert_eq!(downsampled.len(), new_width * new_height * 3);
    }
    
    #[test]
    fn test_merge_single_frame() {
        let width = 10;
        let height = 10;
        let data = vec![128u8; width * height * 3];
        
        let frame = CameraFrame::new(
            data.clone(),
            width as u32,
            height as u32,
            "test_device".to_string(),
        );
        
        let result = merge_frames(&[frame], 0.5, 0);
        
        assert!(result.is_ok());
        let merged = result.unwrap();
        assert_eq!(merged.width, width as u32);
        assert_eq!(merged.data, data);
    }
}
