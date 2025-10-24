/// Focus stacking Tauri commands
/// 
/// Provides commands for capturing and merging focus-stacked images
use tauri::command;
use crate::types::CameraFormat;
use crate::focus_stack::{FocusStackConfig, FocusStackResult};
use crate::focus_stack::capture::{capture_focus_sequence, capture_focus_brackets};
use crate::focus_stack::align::align_frames;
use crate::focus_stack::merge::merge_frames;
use std::time::Instant;

/// Capture and merge a focus stack
#[command]
pub async fn capture_focus_stack(
    device_id: String,
    config: FocusStackConfig,
    format: Option<CameraFormat>,
) -> Result<FocusStackResult, String> {
    log::info!("Starting focus stack capture: device={}, steps={}", 
        device_id, config.num_steps);
    
    let start_time = Instant::now();
    
    // Capture sequence
    let frames = capture_focus_sequence(device_id, config.clone(), format)
        .await
        .map_err(|e| e.to_string())?;
    
    log::info!("Captured {} frames, starting alignment", frames.len());
    
    // Align frames if enabled
    let (aligned_frames, avg_alignment_error) = if config.enable_alignment {
        let alignments = align_frames(&frames)
            .map_err(|e| e.to_string())?;
        
        let avg_error = alignments.iter()
            .map(|a| a.error)
            .sum::<f32>() / alignments.len() as f32;
        
        log::info!("Alignment complete, avg error: {:.3} pixels", avg_error);
        
        // Apply alignment transforms to frames
        let mut aligned = Vec::with_capacity(frames.len());
        for (frame, alignment) in frames.iter().zip(alignments.iter()) {
            let aligned_frame = crate::focus_stack::align::apply_alignment(frame, alignment)
                .map_err(|e| e.to_string())?;
            aligned.push(aligned_frame);
        }
        
        (aligned, avg_error)
    } else {
        (frames, 0.0)
    };
    
    log::info!("Starting merge with {} blend levels", config.blend_levels);
    
    // Merge frames
    let merged_frame = merge_frames(
        &aligned_frames,
        config.sharpness_threshold,
        config.blend_levels,
    ).map_err(|e| e.to_string())?;
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    log::info!("Focus stack complete in {}ms", processing_time_ms);
    
    Ok(FocusStackResult {
        merged_frame,
        num_sources: aligned_frames.len(),
        alignment_error: avg_alignment_error,
        processing_time_ms,
    })
}

/// Capture focus brackets (multiple overlapping focus ranges)
#[command]
pub async fn capture_focus_brackets_command(
    device_id: String,
    brackets: u32,
    shots_per_bracket: u32,
    sharpness_threshold: f32,
    blend_levels: u32,
    format: Option<CameraFormat>,
) -> Result<FocusStackResult, String> {
    log::info!("Starting focus bracket capture: {} brackets x {} shots", 
        brackets, shots_per_bracket);
    
    let start_time = Instant::now();
    
    // Capture all brackets
    let frames = capture_focus_brackets(
        device_id,
        brackets,
        shots_per_bracket,
        format,
    )
    .await
    .map_err(|e| e.to_string())?;
    
    log::info!("Captured {} total frames from brackets", frames.len());
    
    // Align and merge
    let alignments = align_frames(&frames)
        .map_err(|e| e.to_string())?;
    
    let avg_error = alignments.iter()
        .map(|a| a.error)
        .sum::<f32>() / alignments.len() as f32;
    
    let merged_frame = merge_frames(&frames, sharpness_threshold, blend_levels)
        .map_err(|e| e.to_string())?;
    
    let processing_time_ms = start_time.elapsed().as_millis() as u64;
    
    log::info!("Focus bracket stack complete in {}ms", processing_time_ms);
    
    Ok(FocusStackResult {
        merged_frame,
        num_sources: frames.len(),
        alignment_error: avg_error,
        processing_time_ms,
    })
}

/// Get default focus stack configuration
#[command]
pub fn get_default_focus_config() -> FocusStackConfig {
    FocusStackConfig::default()
}

/// Validate focus stack configuration
#[command]
pub fn validate_focus_config(config: FocusStackConfig) -> Result<String, String> {
    if config.num_steps < 2 {
        return Err("num_steps must be at least 2".to_string());
    }
    
    if config.num_steps > 100 {
        return Err("num_steps must be at most 100".to_string());
    }
    
    if config.focus_start < 0.0 || config.focus_start > 1.0 {
        return Err("focus_start must be between 0.0 and 1.0".to_string());
    }
    
    if config.focus_end < 0.0 || config.focus_end > 1.0 {
        return Err("focus_end must be between 0.0 and 1.0".to_string());
    }
    
    if config.sharpness_threshold < 0.0 || config.sharpness_threshold > 1.0 {
        return Err("sharpness_threshold must be between 0.0 and 1.0".to_string());
    }
    
    if config.blend_levels < 3 || config.blend_levels > 10 {
        return Err("blend_levels must be between 3 and 10".to_string());
    }
    
    Ok("Configuration valid".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = get_default_focus_config();
        assert_eq!(config.num_steps, 10);
        assert_eq!(config.blend_levels, 5);
    }
    
    #[test]
    fn test_config_validation_valid() {
        let config = FocusStackConfig::default();
        let result = validate_focus_config(config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_config_validation_invalid_steps() {
        let config = FocusStackConfig {
            num_steps: 1,
            ..Default::default()
        };
        let result = validate_focus_config(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 2"));
    }
    
    #[test]
    fn test_config_validation_invalid_focus_range() {
        let config = FocusStackConfig {
            focus_start: -0.5,
            ..Default::default()
        };
        let result = validate_focus_config(config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_config_validation_invalid_threshold() {
        let config = FocusStackConfig {
            sharpness_threshold: 1.5,
            ..Default::default()
        };
        let result = validate_focus_config(config);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_config_validation_invalid_blend_levels() {
        let config = FocusStackConfig {
            blend_levels: 15,
            ..Default::default()
        };
        let result = validate_focus_config(config);
        assert!(result.is_err());
    }
}
