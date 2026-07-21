use crate::constants::{MIN_RESOLUTION_HEIGHT, MIN_RESOLUTION_WIDTH};
use crate::quality::{BlurDetector, BlurMetrics, ExposureAnalyzer, ExposureMetrics};
use crate::types::CameraFrame;
use serde::{Deserialize, Serialize};

/// Overall quality assessment score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Overall quality (0.0 to 1.0).
    pub overall: f32,
    /// Blur quality score component.
    pub blur: f32,
    /// Exposure quality score component.
    pub exposure: f32,
    /// Composition quality score component.
    pub composition: f32,
    /// Technical quality score component.
    pub technical: f32,
}

impl QualityScore {
    /// Create new quality score with components.
    ///
    /// The overall score is a weighted average of individual components using
    /// the standard `(0.35, 0.35, 0.15, 0.15)` weights.
    #[must_use]
    pub fn new(blur: f32, exposure: f32, composition: f32, technical: f32) -> Self {
        Self::new_weighted(
            blur,
            exposure,
            composition,
            technical,
            (0.35, 0.35, 0.15, 0.15),
        )
    }

    /// Create a quality score with explicit component weights.
    ///
    /// `weights` are `(blur, exposure, composition, technical)`; the overall score
    /// is their normalized weighted average (weights need not sum to 1.0).
    #[must_use]
    pub fn new_weighted(
        blur: f32,
        exposure: f32,
        composition: f32,
        technical: f32,
        weights: (f32, f32, f32, f32),
    ) -> Self {
        // Invariant: Score components must be normalized
        #[cfg(debug_assertions)]
        crate::assert_invariant!(
            (0.0..=1.0).contains(&blur)
                && (0.0..=1.0).contains(&exposure)
                && (0.0..=1.0).contains(&composition)
                && (0.0..=1.0).contains(&technical),
            "Quality components must be normalized 0.0-1.0"
        );

        let total = weights.0 + weights.1 + weights.2 + weights.3;
        let overall = if total > 0.0 {
            (blur * weights.0
                + exposure * weights.1
                + composition * weights.2
                + technical * weights.3)
                / total
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        Self {
            overall,
            blur,
            exposure,
            composition,
            technical,
        }
    }

    /// Check if quality meets minimum threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.overall >= threshold
    }

    /// Get descriptive quality grade.
    #[must_use]
    pub fn get_grade(&self) -> QualityGrade {
        if self.overall >= 0.9 {
            QualityGrade::Excellent
        } else if self.overall >= 0.8 {
            QualityGrade::VeryGood
        } else if self.overall >= 0.7 {
            QualityGrade::Good
        } else if self.overall >= 0.6 {
            QualityGrade::Fair
        } else if self.overall >= 0.4 {
            QualityGrade::Poor
        } else {
            QualityGrade::VeryPoor
        }
    }
}

/// Quality grade enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityGrade {
    /// 0.9+ score. Exceptional quality.
    Excellent,
    /// 0.8-0.89 score. High quality, suitable for professional use.
    VeryGood,
    /// 0.7-0.79 score. Good quality, acceptable for most uses.
    Good,
    /// 0.6-0.69 score. Acceptable but imperfect.
    Fair,
    /// 0.4-0.59 score. Noticeable flaws.
    Poor,
    /// <0.4 score. Unusable quality.
    VeryPoor,
}

impl QualityGrade {
    /// Get string representation for display.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::VeryGood => "Very Good",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
            Self::VeryPoor => "Very Poor",
        }
    }
}

/// Analysis profile controlling the cost/accuracy trade-off for quality validation.
///
/// - `Standard` matches the original behavior (full resolution, moderate noise
///   sampling, balanced weights).
/// - `FastPreview` is for preview / hot paths: downscales the frame, samples noise
///   coarsely, and weights only blur + exposure (skips composition/technical).
/// - `FinalCapture` is the accurate pass: denser noise sampling and slightly more
///   weight on composition/technical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QualityProfile {
    /// Original balanced behavior.
    #[default]
    Standard,
    /// Cheap pass for live preview / hot loops.
    FastPreview,
    /// Accurate pass for final captured frames.
    FinalCapture,
}

impl QualityProfile {
    /// Component weights `(blur, exposure, composition, technical)`.
    pub fn weights(&self) -> (f32, f32, f32, f32) {
        match self {
            QualityProfile::Standard => (0.35, 0.35, 0.15, 0.15),
            QualityProfile::FastPreview => (0.5, 0.5, 0.0, 0.0),
            QualityProfile::FinalCapture => (0.35, 0.35, 0.2, 0.2),
        }
    }

    /// Maximum dimension (px) to analyze; larger frames are downscaled.
    /// `None` disables downscaling.
    pub fn max_analysis_dimension(&self) -> Option<u32> {
        match self {
            QualityProfile::FastPreview => Some(320),
            QualityProfile::Standard | QualityProfile::FinalCapture => None,
        }
    }

    /// Byte stride for noise sampling; smaller = denser (more accurate).
    pub fn noise_sampling_step(&self) -> usize {
        match self {
            QualityProfile::FastPreview => 1200,
            QualityProfile::Standard => 300,
            QualityProfile::FinalCapture => 30,
        }
    }

    /// Default validation thresholds for this profile.
    pub fn default_config(&self) -> ValidationConfig {
        match self {
            QualityProfile::Standard => ValidationConfig::default(),
            QualityProfile::FastPreview => ValidationConfig {
                blur_threshold: 0.4,
                exposure_threshold: 0.4,
                overall_threshold: 0.4,
                min_resolution: (320, 240),
                max_noise_level: 0.4,
            },
            QualityProfile::FinalCapture => ValidationConfig {
                blur_threshold: 0.6,
                exposure_threshold: 0.6,
                overall_threshold: 0.7,
                min_resolution: (MIN_RESOLUTION_WIDTH, MIN_RESOLUTION_HEIGHT),
                max_noise_level: 0.3,
            },
        }
    }
}

/// Comprehensive quality report generated by validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Overall score breakdown.
    pub score: QualityScore,
    /// Textual grade assessment.
    pub grade: QualityGrade,
    /// Detailed blur metrics if available.
    pub blur_metrics: Option<BlurMetrics>,
    /// Detailed exposure metrics if available.
    pub exposure_metrics: Option<ExposureMetrics>,
    /// Quality improvement suggestions.
    pub recommendations: Vec<String>,
    /// Whether the frame passed validation thresholds.
    pub is_acceptable: bool,
    /// Low-level technical details.
    pub technical_details: TechnicalDetails,
}

/// Technical analysis details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDetails {
    /// Frame resolution (width, height).
    pub resolution: (u32, u32),
    /// Total pixel count.
    pub pixel_count: u32,
    /// Aspect ratio (width/height).
    pub aspect_ratio: f32,
    /// Measured noise level.
    pub noise_estimate: f32,
    /// Color distribution analysis.
    pub color_distribution: ColorDistribution,
}

/// Color distribution analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDistribution {
    /// Mean red channel value normalized (0.0-1.0).
    pub red_mean: f32,
    /// Mean green channel value normalized (0.0-1.0).
    pub green_mean: f32,
    /// Mean blue channel value normalized (0.0-1.0).
    pub blue_mean: f32,
    /// Mean saturation (0.0-1.0).
    pub saturation_mean: f32,
    /// White balance score (higher is better).
    pub color_balance_score: f32,
}

/// Quality validation configuration thresholds.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum acceptable blur score.
    pub blur_threshold: f32,
    /// Minimum acceptable exposure score.
    pub exposure_threshold: f32,
    /// Minimum overall quality score.
    pub overall_threshold: f32,
    /// Minimum resolution required.
    pub min_resolution: (u32, u32),
    /// Maximum acceptable noise level.
    pub max_noise_level: f32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            blur_threshold: 0.6,     // Minimum blur quality
            exposure_threshold: 0.6, // Minimum exposure quality
            overall_threshold: 0.7,  // Minimum overall quality
            min_resolution: (MIN_RESOLUTION_WIDTH, MIN_RESOLUTION_HEIGHT), // Minimum resolution (VGA)
            max_noise_level: 0.3, // Maximum acceptable noise
        }
    }
}

/// Quality validator for automated frame assessment
#[derive(Default)]
pub struct QualityValidator {
    blur_detector: BlurDetector,
    exposure_analyzer: ExposureAnalyzer,
    config: ValidationConfig,
    profile: QualityProfile,
}

impl QualityValidator {
    /// Create new quality validator with custom configuration (Standard profile).
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            blur_detector: BlurDetector::default(),
            exposure_analyzer: ExposureAnalyzer::default(),
            config,
            profile: QualityProfile::Standard,
        }
    }

    /// Create a validator using a named analysis profile (applies profile defaults).
    pub fn with_profile(profile: QualityProfile) -> Self {
        Self {
            blur_detector: BlurDetector::default(),
            exposure_analyzer: ExposureAnalyzer::default(),
            config: profile.default_config(),
            profile,
        }
    }

    /// Get the current validation configuration
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Get the active analysis profile.
    pub fn profile(&self) -> QualityProfile {
        self.profile
    }

    /// Validate frame quality comprehensively
    pub fn validate_frame(&self, frame: &CameraFrame) -> QualityReport {
        // Fast-preview profiles downscale large frames before analysis.
        let analyzed = match self.profile.max_analysis_dimension() {
            Some(max_dim) => Self::downscale_frame(frame, max_dim),
            None => frame.clone(),
        };

        // Analyze blur
        let blur_metrics = self.blur_detector.analyze_frame(&analyzed);

        // Analyze exposure
        let exposure_metrics = self.exposure_analyzer.analyze_frame(&analyzed);

        // Analyze composition and technical aspects
        let technical_details =
            Self::analyze_technical_aspects(&analyzed, self.profile.noise_sampling_step());
        let composition_score = self.analyze_composition(&analyzed, &technical_details);

        // Calculate overall quality score using the profile's component weights
        let quality_score = QualityScore::new_weighted(
            blur_metrics.quality_score,
            exposure_metrics.quality_score,
            composition_score,
            // Technical score (inverted noise)
            1.0 - technical_details.noise_estimate,
            self.profile.weights(),
        );

        let grade = quality_score.get_grade();

        // Generate recommendations
        let recommendations =
            self.generate_recommendations(&blur_metrics, &exposure_metrics, &technical_details);

        // Check if acceptable
        let is_acceptable = self.is_frame_acceptable(&quality_score, &technical_details);

        QualityReport {
            score: quality_score,
            grade,
            blur_metrics: Some(blur_metrics),
            exposure_metrics: Some(exposure_metrics),
            recommendations,
            is_acceptable,
            technical_details,
        }
    }

    /// Analyze technical aspects of the frame
    fn analyze_technical_aspects(frame: &CameraFrame, noise_step: usize) -> TechnicalDetails {
        let resolution = (frame.width, frame.height);
        let pixel_count = frame.width * frame.height;
        #[allow(clippy::cast_precision_loss)] // u32 fits in f32 mantissa for typical resolutions
        let aspect_ratio = frame.width as f32 / frame.height as f32;

        // Estimate noise level (sampling density is controlled by the profile)
        let noise_estimate = Self::estimate_noise_level(&frame.data, noise_step);

        // Analyze color distribution
        let color_distribution = Self::analyze_color_distribution(&frame.data);

        TechnicalDetails {
            resolution,
            pixel_count,
            aspect_ratio,
            noise_estimate,
            color_distribution,
        }
    }

    /// Estimate noise level in the image
    ///
    /// `step` is the byte stride between samples (aligned to pixel boundaries).
    /// Smaller strides sample more pixels and yield a more accurate estimate.
    fn estimate_noise_level(rgb_data: &[u8], step: usize) -> f32 {
        if rgb_data.len() < 9 {
            return 1.0; // High noise for very small images
        }

        // Align stride to pixel boundaries (3 bytes) and keep it sane.
        let stride = ((step / 3).max(1)) * 3;

        // Simple noise estimation using local variance
        let mut noise_values = Vec::new();

        // Sample every `stride` bytes to estimate noise
        for i in (0..rgb_data.len()).step_by(stride) {
            // Every 100 pixels * 3 channels
            if i + 8 < rgb_data.len() {
                let r1 = f32::from(rgb_data[i]);
                let g1 = f32::from(rgb_data[i + 1]);
                let b1 = f32::from(rgb_data[i + 2]);

                let r2 = f32::from(rgb_data[i + 3]);
                let g2 = f32::from(rgb_data[i + 4]);
                let b2 = f32::from(rgb_data[i + 5]);

                let r3 = f32::from(rgb_data[i + 6]);
                let g3 = f32::from(rgb_data[i + 7]);
                let b3 = f32::from(rgb_data[i + 8]);

                // Calculate local variance
                let pixels = [
                    (r1 + g1 + b1) / 3.0,
                    (r2 + g2 + b2) / 3.0,
                    (r3 + g3 + b3) / 3.0,
                ];

                let mean = pixels.iter().sum::<f32>() / 3.0;
                let variance = pixels.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / 3.0;

                noise_values.push(variance);
            }
        }

        if noise_values.is_empty() {
            return 0.5;
        }

        #[allow(clippy::cast_precision_loss)] // len() is small; f32 mantissa sufficient
        let mean_noise = noise_values.iter().sum::<f32>() / noise_values.len() as f32;
        (mean_noise / 255.0).clamp(0.0, 1.0)
    }

    /// Analyze color distribution in the image
    fn analyze_color_distribution(rgb_data: &[u8]) -> ColorDistribution {
        if rgb_data.is_empty() {
            return ColorDistribution {
                red_mean: 0.0,
                green_mean: 0.0,
                blue_mean: 0.0,
                saturation_mean: 0.0,
                color_balance_score: 0.0,
            };
        }

        let mut red_sum = 0u64;
        let mut green_sum = 0u64;
        let mut blue_sum = 0u64;
        let mut saturation_sum = 0.0f32;
        let pixel_count = rgb_data.len() / 3;

        for i in (0..rgb_data.len()).step_by(3) {
            let r = f32::from(rgb_data[i]);
            let g = f32::from(rgb_data[i + 1]);
            let b = f32::from(rgb_data[i + 2]);

            red_sum += u64::from(rgb_data[i]);
            green_sum += u64::from(rgb_data[i + 1]);
            blue_sum += u64::from(rgb_data[i + 2]);

            // Calculate saturation (simple method)
            let max_val = r.max(g.max(b));
            let min_val = r.min(g.min(b));
            let saturation = if max_val > 0.0 {
                (max_val - min_val) / max_val
            } else {
                0.0
            };
            saturation_sum += saturation;
        }

        #[allow(clippy::cast_precision_loss)]
        // u64 sum / pixel_count in 0..1e6 range, f32 mantissa sufficient
        let red_mean = red_sum as f32 / (pixel_count as f32 * 255.0);
        #[allow(clippy::cast_precision_loss)]
        // u64 sum / pixel_count in 0..1e6 range, f32 mantissa sufficient
        let green_mean = green_sum as f32 / (pixel_count as f32 * 255.0);
        #[allow(clippy::cast_precision_loss)]
        // u64 sum / pixel_count in 0..1e6 range, f32 mantissa sufficient
        let blue_mean = blue_sum as f32 / (pixel_count as f32 * 255.0);
        #[allow(clippy::cast_precision_loss)]
        // pixel_count in 0..1e6 range, f32 mantissa sufficient
        let saturation_mean = saturation_sum / pixel_count as f32;

        // Calculate color balance score (how balanced are the R, G, B channels)
        let color_means = [red_mean, green_mean, blue_mean];
        let mean_of_means = color_means.iter().sum::<f32>() / 3.0;
        let color_variance = color_means
            .iter()
            .map(|&x| (x - mean_of_means).powi(2))
            .sum::<f32>()
            / 3.0;

        let color_balance_score = (1.0 - color_variance.sqrt()).clamp(0.0, 1.0);

        ColorDistribution {
            red_mean,
            green_mean,
            blue_mean,
            saturation_mean,
            color_balance_score,
        }
    }

    /// Downscale a frame to at most `max_dim` on its longest side using box
    /// average pooling. Used by fast-preview profiles to cut analysis cost.
    fn downscale_frame(frame: &CameraFrame, max_dim: u32) -> CameraFrame {
        let max_side = frame.width.max(frame.height);
        if max_side <= max_dim {
            return frame.clone();
        }

        let factor = max_side.div_ceil(max_dim) as usize;
        let new_w = (frame.width as usize / factor).max(1);
        let new_h = (frame.height as usize / factor).max(1);
        let area = u32::try_from(factor * factor).unwrap_or(u32::MAX);

        let mut out = vec![0u8; new_w * new_h * 3];
        for y in 0..new_h {
            for x in 0..new_w {
                let mut sr = 0u32;
                let mut sg = 0u32;
                let mut sb = 0u32;
                for dy in 0..factor {
                    for dx in 0..factor {
                        let sx = x * factor + dx;
                        let sy = y * factor + dy;
                        if sx < frame.width as usize && sy < frame.height as usize {
                            let idx = (sy * frame.width as usize + sx) * 3;
                            sr += u32::from(frame.data[idx]);
                            sg += u32::from(frame.data[idx + 1]);
                            sb += u32::from(frame.data[idx + 2]);
                        }
                    }
                }
                let dst = (y * new_w + x) * 3;
                out[dst] = u8::try_from(sr / area).unwrap_or(u8::MAX);
                out[dst + 1] = u8::try_from(sg / area).unwrap_or(u8::MAX);
                out[dst + 2] = u8::try_from(sb / area).unwrap_or(u8::MAX);
            }
        }

        CameraFrame::new(
            out,
            u32::try_from(new_w).unwrap_or(u32::MAX),
            u32::try_from(new_h).unwrap_or(u32::MAX),
            frame.device_id.clone(),
        )
        .with_format(frame.format.clone())
    }

    /// Analyze composition quality
    fn analyze_composition(&self, _frame: &CameraFrame, technical: &TechnicalDetails) -> f32 {
        // Composition analysis based on technical quality
        // Future: Add rule-of-thirds detection, subject placement analysis

        // Resolution score
        let resolution_score = if technical.resolution.0 >= self.config.min_resolution.0
            && technical.resolution.1 >= self.config.min_resolution.1
        {
            1.0
        } else {
            0.6
        };

        // Aspect ratio score (prefer standard ratios)
        let aspect_ratio_score = match technical.aspect_ratio {
            ratio if (ratio - 16.0 / 9.0).abs() < 0.1 => 1.0, // 16:9
            ratio if (ratio - 4.0 / 3.0).abs() < 0.1 => 0.9,  // 4:3
            ratio if (ratio - 3.0 / 2.0).abs() < 0.1 => 0.8,  // 3:2
            _ => 0.6,
        };

        // Color balance score
        let color_score = technical.color_distribution.color_balance_score;

        // Noise penalty
        let noise_factor = if technical.noise_estimate > self.config.max_noise_level {
            0.8
        } else {
            1.0
        };

        // Combine scores
        let composition_score =
            (resolution_score * 0.4 + aspect_ratio_score * 0.3 + color_score * 0.3) * noise_factor;

        composition_score.clamp(0.0, 1.0)
    }

    /// Generate quality improvement recommendations
    fn generate_recommendations(
        &self,
        blur_metrics: &BlurMetrics,
        exposure_metrics: &ExposureMetrics,
        technical: &TechnicalDetails,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Blur recommendations
        match blur_metrics.blur_level {
            crate::quality::BlurLevel::Blurry | crate::quality::BlurLevel::VeryBlurry => {
                recommendations.push(
                    "Image is blurry. Try stabilizing the camera or using faster shutter speed."
                        .to_string(),
                );
            }
            _ => {}
        }

        // Exposure recommendations
        match exposure_metrics.exposure_level {
            crate::quality::ExposureLevel::Underexposed => {
                recommendations.push(
                    "Image is underexposed. Increase exposure time, ISO, or add lighting."
                        .to_string(),
                );
            }
            crate::quality::ExposureLevel::Overexposed => {
                recommendations.push(
                    "Image is overexposed. Decrease exposure time, lower ISO, or reduce lighting."
                        .to_string(),
                );
            }
            _ => {}
        }

        // Noise recommendations
        if technical.noise_estimate > self.config.max_noise_level {
            recommendations.push(
                "High noise detected. Consider lowering ISO or improving lighting conditions."
                    .to_string(),
            );
        }

        // Resolution recommendations
        if technical.resolution.0 < self.config.min_resolution.0
            || technical.resolution.1 < self.config.min_resolution.1
        {
            recommendations.push(
                "Low resolution detected. Consider using higher resolution settings.".to_string(),
            );
        }

        // Color balance recommendations
        if technical.color_distribution.color_balance_score < 0.6 {
            recommendations.push(
                "Poor color balance detected. Check white balance settings or lighting conditions."
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations
                .push("Image quality is good. No specific improvements needed.".to_string());
        }

        recommendations
    }

    /// Check if frame meets acceptance criteria
    fn is_frame_acceptable(
        &self,
        quality_score: &QualityScore,
        technical: &TechnicalDetails,
    ) -> bool {
        quality_score.overall >= self.config.overall_threshold
            && quality_score.blur >= self.config.blur_threshold
            && quality_score.exposure >= self.config.exposure_threshold
            && technical.resolution.0 >= self.config.min_resolution.0
            && technical.resolution.1 >= self.config.min_resolution.1
            && technical.noise_estimate <= self.config.max_noise_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_frame(width: u32, height: u32, brightness: u8) -> CameraFrame {
        let size = (width * height * 3) as usize;
        let data = vec![brightness; size];
        CameraFrame::new(data, width, height, "test".to_string())
    }

    #[test]
    fn test_quality_score_creation() {
        let score = QualityScore::new(0.8, 0.9, 0.7, 0.6);

        assert!(score.overall > 0.0 && score.overall <= 1.0);
        assert!((score.blur - 0.8).abs() < 1e-6);
        assert!((score.exposure - 0.9).abs() < 1e-6);
        assert!((score.composition - 0.7).abs() < 1e-6);
        assert!((score.technical - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_quality_grade() {
        let excellent_score = QualityScore::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(excellent_score.get_grade(), QualityGrade::Excellent);

        let poor_score = QualityScore::new(0.3, 0.4, 0.2, 0.5);
        // The actual calculated score might be VeryPoor due to weighted combination
        assert!(matches!(
            poor_score.get_grade(),
            QualityGrade::Poor | QualityGrade::VeryPoor
        ));
    }

    #[test]
    fn test_quality_validator_creation() {
        let validator = QualityValidator::default();
        assert!((validator.config.overall_threshold - 0.7).abs() < 1e-6);

        let custom_config = ValidationConfig {
            blur_threshold: 0.8,
            exposure_threshold: 0.8,
            overall_threshold: 0.9,
            min_resolution: (1920, 1080),
            max_noise_level: 0.2,
        };

        let custom_validator = QualityValidator::new(custom_config);
        assert!((custom_validator.config.overall_threshold - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_frame_validation() {
        let validator = QualityValidator::default();
        let frame = create_test_frame(1280, 720, 128);

        let report = validator.validate_frame(&frame);

        assert!(report.score.overall >= 0.0 && report.score.overall <= 1.0);
        assert!(report.technical_details.pixel_count > 0);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_noise_estimation() {
        let noisy_data = vec![0, 255, 0, 255, 0, 255, 0, 255, 0]; // High noise pattern
        let noise_level = QualityValidator::estimate_noise_level(&noisy_data, 300);

        assert!(noise_level > 0.0 && noise_level <= 1.0);
    }

    #[test]
    fn test_color_distribution_analysis() {
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // Red, Green, Blue
        let color_dist = QualityValidator::analyze_color_distribution(&rgb_data);

        assert!(color_dist.red_mean > 0.0);
        assert!(color_dist.green_mean > 0.0);
        assert!(color_dist.blue_mean > 0.0);
        assert!(color_dist.color_balance_score >= 0.0 && color_dist.color_balance_score <= 1.0);
    }

    #[test]
    fn test_low_resolution_rejection() {
        let config = ValidationConfig {
            min_resolution: (1920, 1080), // Require HD
            ..Default::default()
        };

        let validator = QualityValidator::new(config);
        let low_res_frame = create_test_frame(640, 480, 128);

        let report = validator.validate_frame(&low_res_frame);
        assert!(!report.is_acceptable);

        let recommendations_text = report.recommendations.join(" ");
        assert!(recommendations_text.contains("resolution"));
    }

    #[test]
    fn test_profile_weights_change_overall() {
        let frame = create_test_frame(1280, 720, 128);

        // FastPreview weights only blur + exposure, so composition/technical are ignored.
        let fast = QualityValidator::with_profile(QualityProfile::FastPreview);
        let rf = fast.validate_frame(&frame);
        let expected = f32::midpoint(rf.score.blur, rf.score.exposure);
        assert!((rf.score.overall - expected).abs() < 1e-3);

        // Standard uses all four components and reports a profile of Standard.
        let std = QualityValidator::with_profile(QualityProfile::Standard);
        assert_eq!(std.profile(), QualityProfile::Standard);
        let _ = std.validate_frame(&frame);
    }

    #[test]
    fn test_fast_preview_downscales_analysis() {
        let frame = create_test_frame(1920, 1080, 128);
        let fast = QualityValidator::with_profile(QualityProfile::FastPreview);

        let report = fast.validate_frame(&frame);

        // Downscaled to <=320 on the long side, so the analyzed resolution is small.
        assert!(report.technical_details.resolution.0 <= 320);
        assert_eq!(fast.profile(), QualityProfile::FastPreview);
    }

    #[test]
    fn test_profile_noise_sampling_detects_variation() {
        // Build a frame with deterministic per-pixel luminance variation (noise-like).
        let mut data = vec![128u8; 1920 * 1080 * 3];
        for i in (0..data.len()).step_by(3) {
            let n = u8::try_from(i % 17).unwrap_or(u8::MAX);
            data[i] = data[i].wrapping_add(n);
        }
        let frame = CameraFrame::new(data, 1920, 1080, "test".to_string());

        let fast = QualityValidator::with_profile(QualityProfile::FastPreview);
        let final_v = QualityValidator::with_profile(QualityProfile::FinalCapture);

        let nf = fast.validate_frame(&frame).technical_details.noise_estimate;
        let nc = final_v
            .validate_frame(&frame)
            .technical_details
            .noise_estimate;

        // Both estimates are valid; FinalCapture samples far more pixels so it
        // surfaces the injected variation rather than averaging it away.
        assert!((0.0..=1.0).contains(&nf));
        assert!((0.0..=1.0).contains(&nc));
        assert!(nc > 1e-3, "FinalCapture should detect the injected noise");
    }
}
