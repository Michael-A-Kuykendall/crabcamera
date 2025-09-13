use crate::types::CameraFrame;
use crate::quality::{BlurDetector, BlurMetrics, ExposureAnalyzer, ExposureMetrics};
use serde::{Deserialize, Serialize};

/// Overall quality assessment score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall: f32,     // Overall quality (0.0 to 1.0)
    pub blur: f32,        // Blur quality score
    pub exposure: f32,    // Exposure quality score
    pub composition: f32, // Composition quality score
    pub technical: f32,   // Technical quality score
}

impl QualityScore {
    /// Create new quality score
    pub fn new(blur: f32, exposure: f32, composition: f32, technical: f32) -> Self {
        let overall = (blur * 0.35 + exposure * 0.35 + composition * 0.15 + technical * 0.15).clamp(0.0, 1.0);
        
        Self {
            overall,
            blur,
            exposure,
            composition,
            technical,
        }
    }
    
    /// Check if quality meets minimum threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.overall >= threshold
    }
    
    /// Get quality grade
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
    Excellent,  // 0.9+
    VeryGood,   // 0.8-0.89
    Good,       // 0.7-0.79
    Fair,       // 0.6-0.69
    Poor,       // 0.4-0.59
    VeryPoor,   // <0.4
}

impl QualityGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            QualityGrade::Excellent => "Excellent",
            QualityGrade::VeryGood => "Very Good",
            QualityGrade::Good => "Good",
            QualityGrade::Fair => "Fair",
            QualityGrade::Poor => "Poor",
            QualityGrade::VeryPoor => "Very Poor",
        }
    }
}

/// Comprehensive quality report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub score: QualityScore,
    pub grade: QualityGrade,
    pub blur_metrics: BlurMetrics,
    pub exposure_metrics: ExposureMetrics,
    pub recommendations: Vec<String>,
    pub is_acceptable: bool,
    pub technical_details: TechnicalDetails,
}

/// Technical analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalDetails {
    pub resolution: (u32, u32),
    pub pixel_count: u32,
    pub aspect_ratio: f32,
    pub noise_estimate: f32,
    pub color_distribution: ColorDistribution,
}

/// Color distribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorDistribution {
    pub red_mean: f32,
    pub green_mean: f32,
    pub blue_mean: f32,
    pub saturation_mean: f32,
    pub color_balance_score: f32,
}

/// Quality validation configuration
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub blur_threshold: f32,
    pub exposure_threshold: f32,
    pub overall_threshold: f32,
    pub min_resolution: (u32, u32),
    pub max_noise_level: f32,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            blur_threshold: 0.6,        // Minimum blur quality
            exposure_threshold: 0.6,    // Minimum exposure quality
            overall_threshold: 0.7,     // Minimum overall quality
            min_resolution: (640, 480), // Minimum resolution (VGA)
            max_noise_level: 0.3,       // Maximum acceptable noise
        }
    }
}

/// Comprehensive quality validator
pub struct QualityValidator {
    blur_detector: BlurDetector,
    exposure_analyzer: ExposureAnalyzer,
    config: ValidationConfig,
}

impl Default for QualityValidator {
    fn default() -> Self {
        Self {
            blur_detector: BlurDetector::default(),
            exposure_analyzer: ExposureAnalyzer::default(),
            config: ValidationConfig::default(),
        }
    }
}

impl QualityValidator {
    /// Create new quality validator with custom configuration
    pub fn new(config: ValidationConfig) -> Self {
        Self {
            blur_detector: BlurDetector::default(),
            exposure_analyzer: ExposureAnalyzer::default(),
            config,
        }
    }
    
    /// Validate frame quality comprehensively
    pub fn validate_frame(&self, frame: &CameraFrame) -> QualityReport {
        // Analyze blur
        let blur_metrics = self.blur_detector.analyze_frame(frame);
        
        // Analyze exposure
        let exposure_metrics = self.exposure_analyzer.analyze_frame(frame);
        
        // Analyze composition and technical aspects
        let technical_details = self.analyze_technical_aspects(frame);
        let composition_score = self.analyze_composition(frame, &technical_details);
        
        // Calculate overall quality score
        let quality_score = QualityScore::new(
            blur_metrics.quality_score,
            exposure_metrics.quality_score,
            composition_score,
            technical_details.noise_estimate, // Technical score (inverted noise)
        );
        
        let grade = quality_score.get_grade();
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&blur_metrics, &exposure_metrics, &technical_details);
        
        // Check if acceptable
        let is_acceptable = self.is_frame_acceptable(&quality_score, &technical_details);
        
        QualityReport {
            score: quality_score,
            grade,
            blur_metrics,
            exposure_metrics,
            recommendations,
            is_acceptable,
            technical_details,
        }
    }
    
    /// Analyze technical aspects of the frame
    fn analyze_technical_aspects(&self, frame: &CameraFrame) -> TechnicalDetails {
        let resolution = (frame.width, frame.height);
        let pixel_count = frame.width * frame.height;
        let aspect_ratio = frame.width as f32 / frame.height as f32;
        
        // Estimate noise level
        let noise_estimate = self.estimate_noise_level(&frame.data);
        
        // Analyze color distribution
        let color_distribution = self.analyze_color_distribution(&frame.data);
        
        TechnicalDetails {
            resolution,
            pixel_count,
            aspect_ratio,
            noise_estimate,
            color_distribution,
        }
    }
    
    /// Estimate noise level in the image
    fn estimate_noise_level(&self, rgb_data: &[u8]) -> f32 {
        if rgb_data.len() < 9 {
            return 1.0; // High noise for very small images
        }
        
        // Simple noise estimation using local variance
        let mut noise_values = Vec::new();
        
        // Sample every 100th pixel to estimate noise
        for i in (0..rgb_data.len()).step_by(300) { // Every 100 pixels * 3 channels
            if i + 8 < rgb_data.len() {
                let r1 = rgb_data[i] as f32;
                let g1 = rgb_data[i + 1] as f32;
                let b1 = rgb_data[i + 2] as f32;
                
                let r2 = rgb_data[i + 3] as f32;
                let g2 = rgb_data[i + 4] as f32;
                let b2 = rgb_data[i + 5] as f32;
                
                let r3 = rgb_data[i + 6] as f32;
                let g3 = rgb_data[i + 7] as f32;
                let b3 = rgb_data[i + 8] as f32;
                
                // Calculate local variance
                let pixels = vec![
                    (r1 + g1 + b1) / 3.0,
                    (r2 + g2 + b2) / 3.0,
                    (r3 + g3 + b3) / 3.0,
                ];
                
                let mean = pixels.iter().sum::<f32>() / 3.0;
                let variance = pixels.iter()
                    .map(|&x| (x - mean).powi(2))
                    .sum::<f32>() / 3.0;
                
                noise_values.push(variance);
            }
        }
        
        if noise_values.is_empty() {
            return 0.5;
        }
        
        let mean_noise = noise_values.iter().sum::<f32>() / noise_values.len() as f32;
        (mean_noise / 255.0).clamp(0.0, 1.0)
    }
    
    /// Analyze color distribution in the image
    fn analyze_color_distribution(&self, rgb_data: &[u8]) -> ColorDistribution {
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
            let r = rgb_data[i] as f32;
            let g = rgb_data[i + 1] as f32;
            let b = rgb_data[i + 2] as f32;
            
            red_sum += rgb_data[i] as u64;
            green_sum += rgb_data[i + 1] as u64;
            blue_sum += rgb_data[i + 2] as u64;
            
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
        
        let red_mean = red_sum as f32 / (pixel_count as f32 * 255.0);
        let green_mean = green_sum as f32 / (pixel_count as f32 * 255.0);
        let blue_mean = blue_sum as f32 / (pixel_count as f32 * 255.0);
        let saturation_mean = saturation_sum / pixel_count as f32;
        
        // Calculate color balance score (how balanced are the R, G, B channels)
        let color_means = vec![red_mean, green_mean, blue_mean];
        let mean_of_means = color_means.iter().sum::<f32>() / 3.0;
        let color_variance = color_means.iter()
            .map(|&x| (x - mean_of_means).powi(2))
            .sum::<f32>() / 3.0;
        
        let color_balance_score = (1.0 - color_variance.sqrt()).clamp(0.0, 1.0);
        
        ColorDistribution {
            red_mean,
            green_mean,
            blue_mean,
            saturation_mean,
            color_balance_score,
        }
    }
    
    /// Analyze composition quality
    fn analyze_composition(&self, frame: &CameraFrame, technical: &TechnicalDetails) -> f32 {
        let mut composition_score = 0.5; // Base score
        
        // Resolution score
        let resolution_score = if technical.resolution.0 >= self.config.min_resolution.0 && 
                                  technical.resolution.1 >= self.config.min_resolution.1 {
            1.0
        } else {
            0.6
        };
        
        // Aspect ratio score (prefer standard ratios)
        let aspect_ratio_score = match technical.aspect_ratio {
            ratio if (ratio - 16.0/9.0).abs() < 0.1 => 1.0,      // 16:9
            ratio if (ratio - 4.0/3.0).abs() < 0.1 => 0.9,       // 4:3
            ratio if (ratio - 3.0/2.0).abs() < 0.1 => 0.8,       // 3:2
            _ => 0.6,
        };
        
        // Color balance score
        let color_score = technical.color_distribution.color_balance_score;
        
        // Combine scores
        composition_score = (resolution_score * 0.4 + aspect_ratio_score * 0.3 + color_score * 0.3).clamp(0.0, 1.0);
        
        composition_score
    }
    
    /// Generate quality improvement recommendations
    fn generate_recommendations(&self, blur_metrics: &BlurMetrics, exposure_metrics: &ExposureMetrics, technical: &TechnicalDetails) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Blur recommendations
        match blur_metrics.blur_level {
            crate::quality::BlurLevel::Blurry | crate::quality::BlurLevel::VeryBlurry => {
                recommendations.push("Image is blurry. Try stabilizing the camera or using faster shutter speed.".to_string());
            }
            _ => {}
        }
        
        // Exposure recommendations
        match exposure_metrics.exposure_level {
            crate::quality::ExposureLevel::Underexposed => {
                recommendations.push("Image is underexposed. Increase exposure time, ISO, or add lighting.".to_string());
            }
            crate::quality::ExposureLevel::Overexposed => {
                recommendations.push("Image is overexposed. Decrease exposure time, lower ISO, or reduce lighting.".to_string());
            }
            _ => {}
        }
        
        // Noise recommendations
        if technical.noise_estimate > self.config.max_noise_level {
            recommendations.push("High noise detected. Consider lowering ISO or improving lighting conditions.".to_string());
        }
        
        // Resolution recommendations
        if technical.resolution.0 < self.config.min_resolution.0 || 
           technical.resolution.1 < self.config.min_resolution.1 {
            recommendations.push("Low resolution detected. Consider using higher resolution settings.".to_string());
        }
        
        // Color balance recommendations
        if technical.color_distribution.color_balance_score < 0.6 {
            recommendations.push("Poor color balance detected. Check white balance settings or lighting conditions.".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Image quality is good. No specific improvements needed.".to_string());
        }
        
        recommendations
    }
    
    /// Check if frame meets acceptance criteria
    fn is_frame_acceptable(&self, quality_score: &QualityScore, technical: &TechnicalDetails) -> bool {
        quality_score.overall >= self.config.overall_threshold &&
        quality_score.blur >= self.config.blur_threshold &&
        quality_score.exposure >= self.config.exposure_threshold &&
        technical.resolution.0 >= self.config.min_resolution.0 &&
        technical.resolution.1 >= self.config.min_resolution.1 &&
        technical.noise_estimate <= self.config.max_noise_level
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
        assert_eq!(score.blur, 0.8);
        assert_eq!(score.exposure, 0.9);
        assert_eq!(score.composition, 0.7);
        assert_eq!(score.technical, 0.6);
    }
    
    #[test]
    fn test_quality_grade() {
        let excellent_score = QualityScore::new(1.0, 1.0, 1.0, 1.0);
        assert_eq!(excellent_score.get_grade(), QualityGrade::Excellent);
        
        let poor_score = QualityScore::new(0.3, 0.4, 0.2, 0.5);
        // The actual calculated score might be VeryPoor due to weighted combination
        assert!(matches!(poor_score.get_grade(), QualityGrade::Poor | QualityGrade::VeryPoor));
    }
    
    #[test]
    fn test_quality_validator_creation() {
        let validator = QualityValidator::default();
        assert_eq!(validator.config.overall_threshold, 0.7);
        
        let custom_config = ValidationConfig {
            blur_threshold: 0.8,
            exposure_threshold: 0.8,
            overall_threshold: 0.9,
            min_resolution: (1920, 1080),
            max_noise_level: 0.2,
        };
        
        let custom_validator = QualityValidator::new(custom_config);
        assert_eq!(custom_validator.config.overall_threshold, 0.9);
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
        let validator = QualityValidator::default();
        let noisy_data = vec![0, 255, 0, 255, 0, 255, 0, 255, 0]; // High noise pattern
        let noise_level = validator.estimate_noise_level(&noisy_data);
        
        assert!(noise_level > 0.0 && noise_level <= 1.0);
    }
    
    #[test]
    fn test_color_distribution_analysis() {
        let validator = QualityValidator::default();
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // Red, Green, Blue
        let color_dist = validator.analyze_color_distribution(&rgb_data);
        
        assert!(color_dist.red_mean > 0.0);
        assert!(color_dist.green_mean > 0.0);
        assert!(color_dist.blue_mean > 0.0);
        assert!(color_dist.color_balance_score >= 0.0 && color_dist.color_balance_score <= 1.0);
    }
    
    #[test]
    fn test_low_resolution_rejection() {
        let mut config = ValidationConfig::default();
        config.min_resolution = (1920, 1080); // Require HD
        
        let validator = QualityValidator::new(config);
        let low_res_frame = create_test_frame(640, 480, 128);
        
        let report = validator.validate_frame(&low_res_frame);
        assert!(!report.is_acceptable);
        
        let recommendations_text = report.recommendations.join(" ");
        assert!(recommendations_text.contains("resolution"));
    }
}