use super::FocusStackError;
use crate::constants::{LUMA_B, LUMA_G, LUMA_R, PYRAMID_POOLING_AREA, PYRAMID_POOLING_SIZE};
/// Image merging module for focus stacking
///
/// Merges aligned images by selecting sharp regions from each frame.
/// Uses pyramid blending for smooth transitions between regions.
use crate::types::CameraFrame;

/// Sharpness map for an image
/// Contains per-pixel sharpness scores (0.0 = blurry, 1.0 = sharp)
#[derive(Debug, Clone)]
pub struct SharpnessMap {
    /// Width of the sharpness map in pixels.
    pub width: u32,
    /// Height of the sharpness map in pixels.
    pub height: u32,
    /// Vector of sharpness scores, row-major.
    pub scores: Vec<f32>,
}

/// Merge multiple aligned frames using focus stacking
///
/// For each pixel, selects the value from the sharpest source image.
/// Uses pyramid blending to avoid harsh transitions.
///
/// # Errors
/// Returns a [`FocusStackError::InsufficientImages`] if no frames are provided,
/// or a [`FocusStackError::DimensionMismatch`] if the frames do not all share
/// the same dimensions.
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

    log::info!(
        "Merging {} frames with {} blend levels",
        frames.len(),
        blend_levels
    );

    let reference = &frames[0];
    let width = reference.width;
    let height = reference.height;

    // Validate all frames have same dimensions
    for frame in frames.iter().skip(1) {
        #[cfg(debug_assertions)]
        crate::assert_invariant!(
            frame.width == width && frame.height == height,
            "Focus stack frames must have identical dimensions"
        );

        if frame.width != width || frame.height != height {
            return Err(FocusStackError::DimensionMismatch {
                expected: (width, height),
                got: (frame.width, frame.height),
            });
        }
    }

    // Validate all frames have valid data
    let expected_data_size = (width * height * 3) as usize;
    for frame in frames {
        if frame.data.len() != expected_data_size {
            return Err(FocusStackError::DataCorruption {
                frame_size: frame.data.len(),
                expected_size: expected_data_size,
            });
        }
    }

    // Compute sharpness maps for all frames
    log::debug!("Computing sharpness maps");
    let sharpness_maps: Vec<SharpnessMap> = frames.iter().map(compute_sharpness_map).collect();

    // Create merged frame
    log::debug!("Creating merged frame");
    let merged_data = if blend_levels > 0 {
        merge_with_pyramid_blending(frames, &sharpness_maps, blend_levels)
    } else {
        merge_simple(frames, &sharpness_maps, sharpness_threshold)
    };

    log::info!("Merge complete");

    Ok(
        CameraFrame::new(merged_data, width, height, reference.device_id.clone())
            .with_format(reference.format.clone()),
    )
}

/// Simple merge: pick sharpest pixel from each frame
fn merge_simple(
    frames: &[CameraFrame],
    sharpness_maps: &[SharpnessMap],
    threshold: f32,
) -> Vec<u8> {
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

    merged
}

/// Merge with pyramid blending for smooth transitions
fn merge_with_pyramid_blending(
    frames: &[CameraFrame],
    sharpness_maps: &[SharpnessMap],
    levels: u32,
) -> Vec<u8> {
    let width = frames[0].width as usize;
    let height = frames[0].height as usize;

    log::debug!("Pyramid blending with {levels} levels");

    // Create weight maps (normalized sharpness)
    let weight_maps = create_weight_maps(sharpness_maps);

    // Build Gaussian pyramids for each frame
    log::debug!("Building Gaussian pyramids");
    let gaussian_pyramids: Vec<Vec<(Vec<u8>, usize, usize)>> = frames
        .iter()
        .map(|frame| build_gaussian_pyramid(&frame.data, width, height, levels))
        .collect();

    // Build Laplacian pyramids (signed detail layers)
    log::debug!("Building Laplacian pyramids");
    let laplacian_pyramids: Vec<Vec<(Vec<f32>, usize, usize)>> = gaussian_pyramids
        .iter()
        .map(|pyramid| build_laplacian_pyramid(pyramid))
        .collect();

    // Build weight pyramids
    log::debug!("Building weight pyramids");
    let weight_pyramids: Vec<Vec<Vec<f32>>> = weight_maps
        .iter()
        .map(|weights| build_weight_pyramid(weights, width, height, levels))
        .collect();

    // Blend at each level using per-pixel normalized weights
    log::debug!("Blending pyramids");
    let blended_pyramid = blend_pyramids(&laplacian_pyramids, &weight_pyramids);

    // Reconstruct the merged image from the blended Laplacian pyramid
    log::debug!("Reconstructing from pyramid");
    reconstruct_from_pyramid(&blended_pyramid)
}

/// Compute sharpness map using Laplacian edge detection
fn compute_sharpness_map(frame: &CameraFrame) -> SharpnessMap {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let expected_size = width * height * 3;

    // Validate frame data integrity
    if frame.data.len() < expected_size {
        // Return zero sharpness for corrupted frames
        return SharpnessMap {
            width: u32::try_from(width).unwrap_or(u32::MAX),
            height: u32::try_from(height).unwrap_or(u32::MAX),
            scores: vec![0.0; width * height],
        };
    }

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
            let mut neighbor_count = 0;
            for (dy, dx) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                // Safe: pixel coordinates never reach usize/i32 boundary for real camera frames
                #[allow(clippy::cast_possible_wrap)]
                let ny = i32::try_from(y).unwrap_or(i32::MAX) + dy;
                #[allow(clippy::cast_possible_wrap)]
                let nx = i32::try_from(x).unwrap_or(i32::MAX) + dx;
                #[allow(clippy::cast_possible_wrap)]
                if ny >= 0
                    && ny < i32::try_from(height).unwrap_or(i32::MAX)
                    && nx >= 0
                    && nx < i32::try_from(width).unwrap_or(i32::MAX)
                {
                    let ny = usize::try_from(ny).unwrap_or(0);
                    let nx = usize::try_from(nx).unwrap_or(0);
                    let neighbor_idx = (ny * width + nx) * 3;
                    if neighbor_idx + 2 < frame.data.len() {
                        let neighbor = luminance(&frame.data[neighbor_idx..neighbor_idx + 3]);
                        laplacian += neighbor;
                        neighbor_count += 1;
                    }
                }
            }
            if neighbor_count > 0 {
                // neighbor_count is 0-4, well within f32 precision
                #[allow(clippy::cast_precision_loss)]
                {
                    laplacian = (neighbor_count as f32 * center - laplacian).abs();
                }
            } else {
                laplacian = 0.0;
            }

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

/// Convert RGB to luminance (Rec. 601)
fn luminance(rgb: &[u8]) -> f32 {
    LUMA_R * f32::from(rgb[0]) + LUMA_G * f32::from(rgb[1]) + LUMA_B * f32::from(rgb[2])
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
            // Frame count is small (< 100 typical), well within f32 precision
            #[allow(clippy::cast_precision_loss)]
            let equal_weight = 1.0 / sharpness_maps.len() as f32;
            for weight_map in &mut weight_maps {
                weight_map[pixel_idx] = equal_weight;
            }
        }
    }

    weight_maps
}

/// Build Gaussian pyramid (2x2 average pooling). Each entry is `(data, width, height)`.
fn build_gaussian_pyramid(
    data: &[u8],
    width: usize,
    height: usize,
    levels: u32,
) -> Vec<(Vec<u8>, usize, usize)> {
    let mut pyramid = Vec::with_capacity(levels as usize);
    pyramid.push((data.to_vec(), width, height));

    let mut current_width = width;
    let mut current_height = height;

    for _ in 1..levels {
        let (downsampled, new_width, new_height) = downsample(
            &pyramid
                .last()
                .expect("pyramid non-empty: initial element pushed above")
                .0,
            current_width,
            current_height,
        );
        pyramid.push((downsampled, new_width, new_height));
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
    let new_width = width / PYRAMID_POOLING_SIZE;
    let new_height = height / PYRAMID_POOLING_SIZE;
    let mut downsampled = vec![0u8; new_width * new_height * 3];

    for y in 0..new_height {
        for x in 0..new_width {
            let dst_idx = (y * new_width + x) * 3;

            // Average block
            let mut sum = [0u32; 3];
            for dy in 0..PYRAMID_POOLING_SIZE {
                for dx in 0..PYRAMID_POOLING_SIZE {
                    let src_x = x * PYRAMID_POOLING_SIZE + dx;
                    let src_y = y * PYRAMID_POOLING_SIZE + dy;
                    let src_idx = (src_y * width + src_x) * 3;

                    if src_idx + 2 < data.len() {
                        sum[0] += u32::from(data[src_idx]);
                        sum[1] += u32::from(data[src_idx + 1]);
                        sum[2] += u32::from(data[src_idx + 2]);
                    }
                }
            }

            downsampled[dst_idx] = u8::try_from(sum[0] / PYRAMID_POOLING_AREA).unwrap_or(0);
            downsampled[dst_idx + 1] = u8::try_from(sum[1] / PYRAMID_POOLING_AREA).unwrap_or(0);
            downsampled[dst_idx + 2] = u8::try_from(sum[2] / PYRAMID_POOLING_AREA).unwrap_or(0);
        }
    }

    (downsampled, new_width, new_height)
}

/// Upsample an f32 RGB image from `(src_w, src_h)` to `(dst_w, dst_h)`
/// using bilinear interpolation.
// usize→f32 precision loss acceptable: pixel coords are small (<10000) for pyramid levels
#[allow(clippy::cast_precision_loss)]
fn upsample_f32(data: &[f32], src_w: usize, src_h: usize, dst_w: usize, dst_h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; dst_w * dst_h * 3];

    for y in 0..dst_h {
        for x in 0..dst_w {
            let sx = if dst_w > 1 {
                x as f32 * (src_w as f32 - 1.0) / (dst_w as f32 - 1.0)
            } else {
                0.0
            };
            let sy = if dst_h > 1 {
                y as f32 * (src_h as f32 - 1.0) / (dst_h as f32 - 1.0)
            } else {
                0.0
            };

            // clamp ensures non-negative and in-bounds; floor removes fractional part
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let x0 = sx.floor().clamp(0.0, (src_w - 1) as f32) as usize;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let y0 = sy.floor().clamp(0.0, (src_h - 1) as f32) as usize;
            let x1 = (x0 + 1).min(src_w - 1);
            let y1 = (y0 + 1).min(src_h - 1);

            let fx = (sx - x0 as f32).clamp(0.0, 1.0);
            let fy = (sy - y0 as f32).clamp(0.0, 1.0);

            for c in 0..3 {
                let v00 = data[(y0 * src_w + x0) * 3 + c];
                let v01 = data[(y0 * src_w + x1) * 3 + c];
                let v10 = data[(y1 * src_w + x0) * 3 + c];
                let v11 = data[(y1 * src_w + x1) * 3 + c];

                let top = v00 * (1.0 - fx) + v01 * fx;
                let bottom = v10 * (1.0 - fx) + v11 * fx;
                out[(y * dst_w + x) * 3 + c] = top * (1.0 - fy) + bottom * fy;
            }
        }
    }

    out
}

/// Build the Laplacian pyramid from a Gaussian pyramid.
///
/// Each level holds the detail lost when downsampling:
/// `Laplacian[i] = Gaussian[i] - upsample(Gaussian[i+1])`. The coarsest
/// level is the residual Gaussian. Stored as `(data, width, height)` with
/// signed `f32` values so detail (including negative differences) is kept.
fn build_laplacian_pyramid(gaussian: &[(Vec<u8>, usize, usize)]) -> Vec<(Vec<f32>, usize, usize)> {
    let levels = gaussian.len();
    let mut laplacian = Vec::with_capacity(levels);

    for i in 0..levels.saturating_sub(1) {
        let cur: Vec<f32> = gaussian[i].0.iter().map(|b| f32::from(*b)).collect();
        let (next_f32, next_w, next_h) = (
            gaussian[i + 1]
                .0
                .iter()
                .map(|b| f32::from(*b))
                .collect::<Vec<f32>>(),
            gaussian[i + 1].1,
            gaussian[i + 1].2,
        );
        let upsampled = upsample_f32(&next_f32, next_w, next_h, gaussian[i].1, gaussian[i].2);

        let mut level = vec![0.0f32; cur.len()];
        for (j, c) in cur.iter().enumerate() {
            level[j] = c - upsampled[j];
        }
        laplacian.push((level, gaussian[i].1, gaussian[i].2));
    }

    // Coarsest level: residual Gaussian (no finer level to subtract from)
    laplacian.push((
        gaussian[levels - 1]
            .0
            .iter()
            .map(|b| f32::from(*b))
            .collect(),
        gaussian[levels - 1].1,
        gaussian[levels - 1].2,
    ));

    laplacian
}

/// Build weight pyramid
fn build_weight_pyramid(
    weights: &[f32],
    width: usize,
    height: usize,
    levels: u32,
) -> Vec<Vec<f32>> {
    let mut pyramid = Vec::with_capacity(levels as usize);
    pyramid.push(weights.to_vec());

    let mut current_width = width;
    let mut current_height = height;

    for _ in 1..levels {
        let (downsampled, new_width, new_height) = downsample_weights(
            pyramid
                .last()
                .expect("pyramid non-empty: initial element pushed above"),
            current_width,
            current_height,
        );
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
    let new_width = width / PYRAMID_POOLING_SIZE;
    let new_height = height / PYRAMID_POOLING_SIZE;
    let mut downsampled = vec![0.0; new_width * new_height];

    for y in 0..new_height {
        for x in 0..new_width {
            let dst_idx = y * new_width + x;
            let mut sum = 0.0;

            for dy in 0..PYRAMID_POOLING_SIZE {
                for dx in 0..PYRAMID_POOLING_SIZE {
                    let src_idx =
                        (y * PYRAMID_POOLING_SIZE + dy) * width + (x * PYRAMID_POOLING_SIZE + dx);
                    if src_idx < weights.len() {
                        sum += weights[src_idx];
                    }
                }
            }

            // Pooling area is a small constant (4), f32 precision is sufficient
            #[allow(clippy::cast_precision_loss)]
            {
                downsampled[dst_idx] = sum / (PYRAMID_POOLING_AREA as f32);
            }
        }
    }

    (downsampled, new_width, new_height)
}

/// Blend pyramids using normalized per-pixel weights.
///
/// Laplacian data is stored interleaved RGB (3 `f32` per pixel), so the
/// pixel index into the per-pixel weight map is `pixel_idx / 3`.
fn blend_pyramids(
    laplacians: &[Vec<(Vec<f32>, usize, usize)>],
    weights: &[Vec<Vec<f32>>],
) -> Vec<(Vec<f32>, usize, usize)> {
    let num_levels = laplacians[0].len();
    let mut blended = Vec::with_capacity(num_levels);

    for level in 0..num_levels {
        let (ref level_data, w, h) = laplacians[0][level];
        let level_size = level_data.len();
        let mut blended_level = vec![0.0f32; level_size];

        for (pixel_idx, blended_pixel) in blended_level.iter_mut().enumerate() {
            let weight_idx = pixel_idx / 3;
            let mut sum = 0.0;
            for frame_idx in 0..laplacians.len() {
                let pixel_val = laplacians[frame_idx][level].0[pixel_idx];
                let weight = weights[frame_idx][level]
                    .get(weight_idx)
                    .copied()
                    .unwrap_or(0.0);
                sum += pixel_val * weight;
            }
            *blended_pixel = sum;
        }

        blended.push((blended_level, w, h));
    }

    blended
}

/// Reconstruct the merged image from a blended Laplacian pyramid.
///
/// Collapses coarse-to-fine: each level is `upsample(reconstruction of the
/// coarser level) + blended detail at that level`, then clamps to `u8`.
fn reconstruct_from_pyramid(pyramid: &[(Vec<f32>, usize, usize)]) -> Vec<u8> {
    let levels = pyramid.len();
    let mut current = pyramid[levels - 1].0.clone();
    let mut current_w = pyramid[levels - 1].1;
    let mut current_h = pyramid[levels - 1].2;

    for level in (0..levels.saturating_sub(1)).rev() {
        let (target_w, target_h) = (pyramid[level].1, pyramid[level].2);
        let upsampled = upsample_f32(&current, current_w, current_h, target_w, target_h);

        let mut reconstructed = vec![0.0f32; target_w * target_h * 3];
        for (j, recon) in reconstructed.iter_mut().enumerate() {
            *recon = pyramid[level].0[j] + upsampled[j];
        }
        current = reconstructed;
        current_w = target_w;
        current_h = target_h;
    }

    // Clamp to [0, 255] guarantees value fits in u8
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    current.iter().map(|v| v.clamp(0.0, 255.0) as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_frame(width: u32, height: u32, value: u8) -> CameraFrame {
        CameraFrame::new(
            vec![value; (width * height * 3) as usize],
            width,
            height,
            "test_device".to_string(),
        )
    }

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
            u32::try_from(width).unwrap_or(u32::MAX),
            u32::try_from(height).unwrap_or(u32::MAX),
            "test_device".to_string(),
        );

        let sharpness = compute_sharpness_map(&frame);

        assert_eq!(sharpness.width, u32::try_from(width).unwrap_or(u32::MAX));
        assert_eq!(sharpness.height, u32::try_from(height).unwrap_or(u32::MAX));
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
            u32::try_from(width).unwrap_or(u32::MAX),
            u32::try_from(height).unwrap_or(u32::MAX),
            "test_device".to_string(),
        );

        let result = merge_frames(&[frame], 0.5, 0);

        assert!(result.is_ok());
        let merged = result.expect("merge expected");
        assert_eq!(merged.width, u32::try_from(width).unwrap_or(u32::MAX));
        assert_eq!(merged.data, data);
    }

    #[test]
    fn test_merge_frames_empty_errors() {
        let empty = merge_frames(&[], 0.5, 0);
        assert!(matches!(
            empty,
            Err(FocusStackError::InsufficientImages { .. })
        ));
    }

    #[test]
    #[should_panic(expected = "Focus stack frames must have identical dimensions")]
    fn test_merge_frames_dimension_mismatch_triggers_invariant_in_debug() {
        let a = mk_frame(8, 8, 100);
        let b = mk_frame(9, 8, 120);
        let _ = merge_frames(&[a, b], 0.5, 0);
    }

    #[test]
    fn test_merge_frames_data_corruption_error() {
        let mut bad = mk_frame(8, 8, 100);
        bad.data.truncate(10);
        let good = mk_frame(8, 8, 120);

        let result = merge_frames(&[bad, good], 0.5, 0);
        assert!(matches!(
            result,
            Err(FocusStackError::DataCorruption { .. })
        ));
    }

    #[test]
    fn test_merge_simple_and_weight_map_helpers() {
        let a = mk_frame(4, 4, 10);
        let b = mk_frame(4, 4, 240);
        let sa = compute_sharpness_map(&a);
        let sb = compute_sharpness_map(&b);

        let merged = merge_simple(&[a.clone(), b.clone()], &[sa.clone(), sb.clone()], 0.0);
        assert_eq!(merged.len(), (4 * 4 * 3) as usize);

        let weights = create_weight_maps(&[sa, sb]);
        assert_eq!(weights.len(), 2);
        assert_eq!(weights[0].len(), 16);
    }

    #[test]
    fn test_compute_sharpness_map_handles_short_data() {
        let mut frame = mk_frame(4, 4, 100);
        frame.data.truncate(5);

        let sharp = compute_sharpness_map(&frame);
        assert_eq!(sharp.width, 4);
        assert_eq!(sharp.height, 4);
        assert_eq!(sharp.scores.len(), 16);
        assert!(sharp.scores.iter().all(|v| (*v - 0.0).abs() < f32::EPSILON));
    }

    #[test]
    fn test_pyramid_pipeline_helpers() {
        let width = 8;
        let height = 8;
        let data = vec![100u8; width * height * 3];

        let gp = build_gaussian_pyramid(&data, width, height, 3);
        assert!(!gp.is_empty());

        let lp = build_laplacian_pyramid(&gp);
        assert_eq!(lp.len(), gp.len());

        let weights = vec![0.5f32; width * height];
        let wp = build_weight_pyramid(&weights, width, height, 3);
        assert!(!wp.is_empty());

        let blended = blend_pyramids(std::slice::from_ref(&lp), std::slice::from_ref(&wp));
        assert!(!blended.is_empty());

        let reconstructed = reconstruct_from_pyramid(&blended);
        assert_eq!(reconstructed.len(), blended[0].0.len());
    }

    #[test]
    fn test_merge_with_pyramid_blending_path() {
        let a = mk_frame(8, 8, 100);
        let b = mk_frame(8, 8, 120);

        let result = merge_frames(&[a, b], 0.3, 3).expect("pyramid merge should succeed");
        assert_eq!(result.width, 8);
        assert_eq!(result.height, 8);
    }
}
