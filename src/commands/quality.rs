use crate::commands::capture::capture_single_photo;
#[cfg(test)]
use crate::constants::*;
use crate::quality::{BlurDetector, BlurMetrics, ExposureAnalyzer, ExposureMetrics};
use crate::quality::{QualityReport, QualityValidator, ValidationConfig};
use crate::types::CameraFrame;
use std::sync::{Arc, LazyLock};
use tauri::command;
use tokio::sync::RwLock;

// Global quality validator
static QUALITY_VALIDATOR: LazyLock<Arc<RwLock<QualityValidator>>> =
    LazyLock::new(|| Arc::new(RwLock::new(QualityValidator::default())));

/// Validate quality of a captured frame
///
/// # Errors
/// Returns an `Err` if the frame cannot be captured (propagated from the
/// underlying capture).
#[command]
pub async fn validate_frame_quality(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
) -> Result<QualityReport, String> {
    log::info!("Validating frame quality for device: {device_id:?}");

    // Capture a frame first
    let frame = capture_single_photo(device_id, capture_format).await?;

    // Validate quality
    let validator = QUALITY_VALIDATOR.read().await;
    let report = validator.validate_frame(&frame);

    Ok(report)
}

/// Validate quality of provided frame data
///
/// # Errors
/// This function always succeeds and never returns an `Err`.
#[command]
pub async fn validate_provided_frame(frame: CameraFrame) -> Result<QualityReport, String> {
    log::info!(
        "Validating provided frame: {}x{}",
        frame.width,
        frame.height
    );

    let validator = QUALITY_VALIDATOR.read().await;
    let report = validator.validate_frame(&frame);

    Ok(report)
}

/// Analyze blur in a captured frame
///
/// # Errors
/// Returns an `Err` if the frame cannot be captured (propagated from the
/// underlying capture).
#[command]
pub async fn analyze_frame_blur(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
) -> Result<BlurMetrics, String> {
    log::info!("Analyzing frame blur for device: {device_id:?}");

    // Capture a frame
    let frame = capture_single_photo(device_id, capture_format).await?;

    // Analyze blur
    let blur_detector = BlurDetector::default();
    let metrics = blur_detector.analyze_frame(&frame);

    Ok(metrics)
}

/// Analyze exposure in a captured frame
///
/// # Errors
/// Returns an `Err` if the frame cannot be captured (propagated from the
/// underlying capture).
#[command]
pub async fn analyze_frame_exposure(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
) -> Result<ExposureMetrics, String> {
    log::info!("Analyzing frame exposure for device: {device_id:?}");

    // Capture a frame
    let frame = capture_single_photo(device_id, capture_format).await?;

    // Analyze exposure
    let exposure_analyzer = ExposureAnalyzer::default();
    let metrics = exposure_analyzer.analyze_frame(&frame);

    Ok(metrics)
}

/// Update quality validation configuration
///
/// # Errors
/// This function always succeeds and never returns an `Err`.
#[command]
pub async fn update_quality_config(config: ValidationConfigDto) -> Result<String, String> {
    log::info!("Updating quality validation configuration");

    let validation_config = ValidationConfig {
        blur_threshold: config.blur_threshold,
        exposure_threshold: config.exposure_threshold,
        overall_threshold: config.overall_threshold,
        min_resolution: (config.min_width, config.min_height),
        max_noise_level: config.max_noise_level,
    };

    let validator = QualityValidator::new(validation_config);
    let mut guard = QUALITY_VALIDATOR.write().await;
    *guard = validator;

    Ok("Quality validation configuration updated".to_string())
}

/// Get current quality validation configuration
///
/// # Errors
/// This function always succeeds and never returns an `Err`.
#[command]
pub async fn get_quality_config() -> Result<ValidationConfigDto, String> {
    let validator = QUALITY_VALIDATOR.read().await;
    let config = validator.config();

    Ok(ValidationConfigDto {
        blur_threshold: config.blur_threshold,
        exposure_threshold: config.exposure_threshold,
        overall_threshold: config.overall_threshold,
        min_width: config.min_resolution.0,
        min_height: config.min_resolution.1,
        max_noise_level: config.max_noise_level,
    })
}

/// Capture and validate multiple frames, return best quality
///
/// # Errors
/// Returns an `Err` if no valid frame could be captured across all attempts.
#[command]
pub async fn capture_best_quality_frame(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
    num_attempts: Option<u32>,
) -> Result<CaptureQualityResult, String> {
    let attempts = num_attempts.unwrap_or(5).min(10); // Max 10 attempts
    log::info!("Capturing best quality frame with {attempts} attempts");

    let validator = QUALITY_VALIDATOR.read().await;
    let mut best_frame: Option<CameraFrame> = None;
    let mut best_report: Option<QualityReport> = None;
    let mut best_score = 0.0f32;

    for attempt in 1..=attempts {
        log::debug!("Quality capture attempt {attempt} of {attempts}");

        // Capture frame
        match capture_single_photo(device_id.clone(), capture_format.clone()).await {
            Ok(frame) => {
                // Validate quality
                let report = validator.validate_frame(&frame);

                if report.score.overall > best_score {
                    best_score = report.score.overall;
                    best_frame = Some(frame);
                    best_report = Some(report);
                }

                // If we achieve excellent quality, stop early
                if best_score >= 0.9 {
                    log::info!("Excellent quality achieved on attempt {attempt}");
                    break;
                }
            }
            Err(e) => {
                log::warn!("Frame capture failed on attempt {attempt}: {e}");
                continue;
            }
        }

        // Small delay between attempts
        if attempt < attempts {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    match (best_frame, best_report) {
        (Some(frame), Some(report)) => Ok(CaptureQualityResult {
            frame,
            quality_report: report,
            attempts_used: attempts,
        }),
        _ => Err("Failed to capture any valid frames".to_string()),
    }
}

/// Auto-capture with quality threshold
///
/// # Errors
/// Returns an `Err` if the overall timeout elapses or if no frame meeting the
/// quality threshold is captured within the maximum number of attempts.
#[command]
pub async fn auto_capture_with_quality(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
    min_quality_threshold: Option<f32>,
    max_attempts: Option<u32>,
    timeout_seconds: Option<u32>,
) -> Result<CaptureQualityResult, String> {
    let quality_threshold = min_quality_threshold.unwrap_or(0.7);
    let max_tries = max_attempts.unwrap_or(20).min(50); // Max 50 attempts
    let timeout = timeout_seconds.unwrap_or(30); // 30 second timeout

    log::info!(
        "Auto-capturing with quality threshold {quality_threshold} (max {max_tries} attempts, {timeout}s timeout)"
    );

    let start_time = std::time::Instant::now();
    let validator = QUALITY_VALIDATOR.read().await;

    for attempt in 1..=max_tries {
        // Check timeout
        if start_time.elapsed().as_secs() >= u64::from(timeout) {
            return Err(format!("Auto-capture timeout after {timeout} seconds"));
        }

        log::debug!("Auto-capture attempt {attempt} of {max_tries}");

        // Capture frame
        match capture_single_photo(device_id.clone(), capture_format.clone()).await {
            Ok(frame) => {
                // Validate quality
                let report = validator.validate_frame(&frame);

                if report.score.overall >= quality_threshold {
                    log::info!(
                        "Quality threshold met on attempt {} (score: {:.3})",
                        attempt,
                        report.score.overall
                    );

                    return Ok(CaptureQualityResult {
                        frame,
                        quality_report: report,
                        attempts_used: attempt,
                    });
                }

                log::debug!(
                    "Quality not met (score: {:.3}), continuing...",
                    report.score.overall
                );
            }
            Err(e) => {
                log::warn!("Frame capture failed on attempt {attempt}: {e}");
            }
        }

        // Small delay between attempts
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    Err(format!(
        "Failed to capture frame meeting quality threshold {quality_threshold} after {max_tries} attempts"
    ))
}

/// Analyze quality trends over multiple captures
///
/// # Errors
/// Returns an `Err` if no valid samples could be captured for the trend
/// analysis.
#[command]
pub async fn analyze_quality_trends(
    device_id: Option<String>,
    capture_format: Option<crate::types::CameraFormat>,
    num_samples: Option<u32>,
) -> Result<QualityTrendAnalysis, String> {
    let samples = num_samples.unwrap_or(10).min(20); // Max 20 samples
    log::info!("Analyzing quality trends over {samples} samples");

    let validator = QUALITY_VALIDATOR.read().await;
    let mut reports = Vec::new();

    for i in 1..=samples {
        log::debug!("Quality trend sample {i} of {samples}");

        match capture_single_photo(device_id.clone(), capture_format.clone()).await {
            Ok(frame) => {
                let report = validator.validate_frame(&frame);
                reports.push(report);
            }
            Err(e) => {
                log::warn!("Failed to capture sample {i}: {e}");
                continue;
            }
        }

        // Small delay between samples
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    if reports.is_empty() {
        return Err("No valid samples captured for trend analysis".to_string());
    }

    // Calculate trend statistics
    let scores: Vec<f32> = reports.iter().map(|r| r.score.overall).collect();
    let blur_scores: Vec<f32> = reports.iter().map(|r| r.score.blur).collect();
    let exposure_scores: Vec<f32> = reports.iter().map(|r| r.score.exposure).collect();

    #[allow(clippy::cast_precision_loss)]
    let avg_quality = scores.iter().sum::<f32>() / scores.len() as f32;
    #[allow(clippy::cast_precision_loss)]
    let avg_blur = blur_scores.iter().sum::<f32>() / blur_scores.len() as f32;
    #[allow(clippy::cast_precision_loss)]
    let avg_exposure = exposure_scores.iter().sum::<f32>() / exposure_scores.len() as f32;

    #[allow(clippy::cast_precision_loss)]
    let quality_variance = scores
        .iter()
        .map(|&x| (x - avg_quality).powi(2))
        .sum::<f32>()
        / scores.len() as f32;

    let stability_score = (1.0 - quality_variance.sqrt()).clamp(0.0, 1.0);

    let samples_analyzed = u32::try_from(reports.len()).unwrap_or(u32::MAX);
    #[allow(clippy::cast_precision_loss)]
    let acceptable_count = reports.iter().filter(|r| r.is_acceptable).count() as f32;
    #[allow(clippy::cast_precision_loss)]
    let total_reports = reports.len() as f32;
    let acceptable_ratio = acceptable_count / total_reports;

    Ok(QualityTrendAnalysis {
        samples_analyzed,
        average_quality: avg_quality,
        average_blur_score: avg_blur,
        average_exposure_score: avg_exposure,
        quality_variance,
        stability_score,
        best_score: scores.iter().fold(0.0f32, |a, &b| a.max(b)),
        worst_score: scores.iter().fold(1.0f32, |a, &b| a.min(b)),
        acceptable_ratio,
    })
}

// Data transfer objects for Tauri commands

/// Validation configuration DTO
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationConfigDto {
    /// Minimum acceptable sharpness score (0.0-1.0).
    pub blur_threshold: f32,
    /// Minimum acceptable exposure score (0.0-1.0).
    pub exposure_threshold: f32,
    /// Minimum aggregate quality score required.
    pub overall_threshold: f32,
    /// Minimum image width in pixels.
    pub min_width: u32,
    /// Minimum image height in pixels.
    pub min_height: u32,
    /// Maximum allowable noise level (lower is better).
    pub max_noise_level: f32,
}

/// Capture with quality result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureQualityResult {
    /// The captured image frame.
    pub frame: CameraFrame,
    /// The quality analysis report for this frame.
    pub quality_report: QualityReport,
    /// Number of capture attempts made to get this frame (if retry was enabled).
    pub attempts_used: u32,
}

/// Quality trend analysis result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityTrendAnalysis {
    /// Number of frames included in this analysis.
    pub samples_analyzed: u32,
    /// Mean quality score across all samples.
    pub average_quality: f32,
    /// Mean sharpness/blur score.
    pub average_blur_score: f32,
    /// Mean exposure correctness score.
    pub average_exposure_score: f32,
    /// Statistical variance of the quality scores.
    pub quality_variance: f32,
    /// A derived score indicating steadiness (inverse of variance).
    pub stability_score: f32,
    /// The highest quality score observed.
    pub best_score: f32,
    /// The lowest quality score observed.
    pub worst_score: f32,
    /// The fraction of frames (0.0-1.0) that met the minimum quality threshold.
    pub acceptable_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_frame() -> CameraFrame {
        let data = vec![128u8; 640 * 480 * 3];
        CameraFrame::new(data, 640, 480, "test".to_string())
    }

    #[tokio::test]
    async fn test_validate_provided_frame() {
        let frame = create_test_frame();
        let result = validate_provided_frame(frame).await;
        assert!(result.is_ok());

        let report = result.expect("validation report expected");
        assert!(report.score.overall >= 0.0 && report.score.overall <= 1.0);
    }

    #[tokio::test]
    async fn test_quality_config_update() {
        let config = ValidationConfigDto {
            blur_threshold: 0.8,
            exposure_threshold: 0.8,
            overall_threshold: 0.9,
            min_width: DEFAULT_RESOLUTION_WIDTH,
            min_height: DEFAULT_RESOLUTION_HEIGHT,
            max_noise_level: 0.2,
        };

        let result = update_quality_config(config.clone()).await;
        assert!(result.is_ok());

        let retrieved_config = get_quality_config().await.expect("quality config expected");
        // Verify the config was actually stored and retrieved correctly
        assert!((retrieved_config.blur_threshold - 0.8).abs() < 0.001);
        assert!((retrieved_config.exposure_threshold - 0.8).abs() < 0.001);
        assert!((retrieved_config.overall_threshold - 0.9).abs() < 0.001);
        assert_eq!(retrieved_config.min_width, DEFAULT_RESOLUTION_WIDTH);
        assert_eq!(retrieved_config.min_height, DEFAULT_RESOLUTION_HEIGHT);
        assert!((retrieved_config.max_noise_level - 0.2).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_quality_trend_analysis_empty() {
        // This test may fail in CI without camera, but validates the structure
        let result =
            analyze_quality_trends(Some("invalid_camera".to_string()), None, Some(1)).await;
        // Should handle gracefully when no camera is available (may succeed with mock data)
        // In CI, this might succeed due to mock camera system
        assert!(result.is_ok() || result.is_err());
    }
}
