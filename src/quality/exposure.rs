use crate::types::CameraFrame;
use serde::{Deserialize, Serialize};

/// Exposure analysis levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExposureLevel {
    Underexposed,   // Too dark
    SlightlyDark,   // Slightly underexposed but acceptable
    WellExposed,    // Optimal exposure
    SlightlyBright, // Slightly overexposed but acceptable
    Overexposed,    // Too bright
}

impl ExposureLevel {
    /// Convert brightness to exposure level
    pub fn from_brightness(brightness: f32) -> Self {
        if brightness < 0.2 {
            ExposureLevel::Underexposed
        } else if brightness < 0.35 {
            ExposureLevel::SlightlyDark
        } else if brightness < 0.65 {
            ExposureLevel::WellExposed
        } else if brightness < 0.8 {
            ExposureLevel::SlightlyBright
        } else {
            ExposureLevel::Overexposed
        }
    }
    
    /// Get quality score (0.0 to 1.0)
    pub fn quality_score(&self) -> f32 {
        match self {
            ExposureLevel::WellExposed => 1.0,
            ExposureLevel::SlightlyDark | ExposureLevel::SlightlyBright => 0.8,
            ExposureLevel::Underexposed | ExposureLevel::Overexposed => 0.3,
        }
    }
}

/// Exposure analysis metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureMetrics {
    pub mean_brightness: f32,      // Average brightness (0.0 to 1.0)
    pub brightness_std: f32,       // Standard deviation of brightness
    pub histogram: Vec<u32>,       // 256-bin brightness histogram
    pub dark_pixel_ratio: f32,     // Ratio of very dark pixels
    pub bright_pixel_ratio: f32,   // Ratio of very bright pixels
    pub dynamic_range: f32,        // Difference between min and max brightness
    pub exposure_level: ExposureLevel, // Overall exposure assessment
    pub quality_score: f32,        // Quality score (0.0 to 1.0)
}

/// Exposure analyzer for image quality assessment
pub struct ExposureAnalyzer {
    dark_threshold: u8,     // Threshold for dark pixels
    bright_threshold: u8,   // Threshold for bright pixels
}

impl Default for ExposureAnalyzer {
    fn default() -> Self {
        Self {
            dark_threshold: 30,   // Pixels below this are considered dark
            bright_threshold: 225, // Pixels above this are considered bright
        }
    }
}

impl ExposureAnalyzer {
    /// Create new exposure analyzer with custom thresholds
    pub fn new(dark_threshold: u8, bright_threshold: u8) -> Self {
        Self {
            dark_threshold,
            bright_threshold,
        }
    }
    
    /// Analyze frame exposure
    pub fn analyze_frame(&self, frame: &CameraFrame) -> ExposureMetrics {
        // Convert to grayscale for luminance analysis
        let grayscale = self.rgb_to_luminance(&frame.data, frame.width, frame.height);
        
        // Calculate histogram
        let histogram = self.calculate_histogram(&grayscale);
        
        // Calculate brightness statistics
        let mean_brightness = self.calculate_mean_brightness(&grayscale);
        let brightness_std = self.calculate_brightness_std(&grayscale, mean_brightness);
        
        // Calculate pixel ratios
        let dark_pixel_ratio = self.calculate_dark_pixel_ratio(&grayscale);
        let bright_pixel_ratio = self.calculate_bright_pixel_ratio(&grayscale);
        
        // Calculate dynamic range
        let dynamic_range = self.calculate_dynamic_range(&histogram);
        
        // Determine exposure level
        let exposure_level = ExposureLevel::from_brightness(mean_brightness);
        let quality_score = self.calculate_quality_score(&exposure_level, brightness_std, dynamic_range);
        
        ExposureMetrics {
            mean_brightness,
            brightness_std,
            histogram,
            dark_pixel_ratio,
            bright_pixel_ratio,
            dynamic_range,
            exposure_level,
            quality_score,
        }
    }
    
    /// Convert RGB to luminance using standard weights
    fn rgb_to_luminance(&self, rgb_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let mut luminance = Vec::with_capacity((width * height) as usize);
        
        for i in (0..rgb_data.len()).step_by(3) {
            let r = rgb_data[i] as f32;
            let g = rgb_data[i + 1] as f32;
            let b = rgb_data[i + 2] as f32;
            
            // ITU-R BT.709 luminance weights
            let y = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
            luminance.push(y);
        }
        
        luminance
    }
    
    /// Calculate 256-bin histogram
    fn calculate_histogram(&self, luminance: &[u8]) -> Vec<u32> {
        let mut histogram = vec![0u32; 256];
        
        for &pixel in luminance {
            histogram[pixel as usize] += 1;
        }
        
        histogram
    }
    
    /// Calculate mean brightness (0.0 to 1.0)
    fn calculate_mean_brightness(&self, luminance: &[u8]) -> f32 {
        if luminance.is_empty() {
            return 0.0;
        }
        
        let sum: u64 = luminance.iter().map(|&x| x as u64).sum();
        (sum as f32) / (luminance.len() as f32 * 255.0)
    }
    
    /// Calculate brightness standard deviation
    fn calculate_brightness_std(&self, luminance: &[u8], mean: f32) -> f32 {
        if luminance.is_empty() {
            return 0.0;
        }
        
        let mean_255 = mean * 255.0; // Convert back to 0-255 scale
        let variance: f32 = luminance.iter()
            .map(|&x| (x as f32 - mean_255).powi(2))
            .sum::<f32>() / luminance.len() as f32;
        
        variance.sqrt() / 255.0 // Normalize to 0-1 scale
    }
    
    /// Calculate ratio of dark pixels
    fn calculate_dark_pixel_ratio(&self, luminance: &[u8]) -> f32 {
        if luminance.is_empty() {
            return 0.0;
        }
        
        let dark_count = luminance.iter()
            .filter(|&&x| x < self.dark_threshold)
            .count();
        
        dark_count as f32 / luminance.len() as f32
    }
    
    /// Calculate ratio of bright pixels
    fn calculate_bright_pixel_ratio(&self, luminance: &[u8]) -> f32 {
        if luminance.is_empty() {
            return 0.0;
        }
        
        let bright_count = luminance.iter()
            .filter(|&&x| x > self.bright_threshold)
            .count();
        
        bright_count as f32 / luminance.len() as f32
    }
    
    /// Calculate dynamic range
    fn calculate_dynamic_range(&self, histogram: &[u32]) -> f32 {
        let mut min_value = 255;
        let mut max_value = 0;
        
        // Find minimum non-zero bin
        for (i, &count) in histogram.iter().enumerate() {
            if count > 0 && i < min_value {
                min_value = i;
                break;
            }
        }
        
        // Find maximum non-zero bin
        for (i, &count) in histogram.iter().enumerate().rev() {
            if count > 0 && i > max_value {
                max_value = i;
                break;
            }
        }
        
        if max_value > min_value {
            (max_value - min_value) as f32 / 255.0
        } else {
            0.0
        }
    }
    
    /// Calculate overall quality score
    fn calculate_quality_score(&self, exposure_level: &ExposureLevel, brightness_std: f32, dynamic_range: f32) -> f32 {
        let exposure_score = exposure_level.quality_score();
        
        // Bonus for good contrast (standard deviation)
        let contrast_score = if brightness_std > 0.15 && brightness_std < 0.35 {
            1.0
        } else if brightness_std > 0.1 && brightness_std < 0.4 {
            0.8
        } else {
            0.5
        };
        
        // Bonus for good dynamic range
        let range_score = if dynamic_range > 0.7 {
            1.0
        } else if dynamic_range > 0.5 {
            0.8
        } else if dynamic_range > 0.3 {
            0.6
        } else {
            0.4
        };
        
        // Weighted combination
        (exposure_score * 0.6 + contrast_score * 0.25 + range_score * 0.15).clamp(0.0, 1.0)
    }
    
    /// Check if exposure is acceptable
    pub fn is_acceptable_exposure(&self, metrics: &ExposureMetrics) -> bool {
        matches!(metrics.exposure_level, 
                 ExposureLevel::WellExposed | 
                 ExposureLevel::SlightlyDark | 
                 ExposureLevel::SlightlyBright)
    }
    
    /// Get exposure correction recommendation
    pub fn get_exposure_correction(&self, metrics: &ExposureMetrics) -> ExposureCorrection {
        match metrics.exposure_level {
            ExposureLevel::Underexposed => ExposureCorrection::IncreaseExposure(1.5),
            ExposureLevel::SlightlyDark => ExposureCorrection::IncreaseExposure(1.2),
            ExposureLevel::WellExposed => ExposureCorrection::NoChange,
            ExposureLevel::SlightlyBright => ExposureCorrection::DecreaseExposure(0.8),
            ExposureLevel::Overexposed => ExposureCorrection::DecreaseExposure(0.6),
        }
    }
}

/// Exposure correction recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExposureCorrection {
    NoChange,
    IncreaseExposure(f32), // Multiplier for exposure time
    DecreaseExposure(f32), // Multiplier for exposure time
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame_with_brightness(width: u32, height: u32, brightness: u8) -> CameraFrame {
        let size = (width * height * 3) as usize;
        let data = vec![brightness; size];
        CameraFrame::new(data, width, height, "test".to_string())
    }
    
    #[test]
    fn test_exposure_level_from_brightness() {
        assert_eq!(ExposureLevel::from_brightness(0.1), ExposureLevel::Underexposed);
        assert_eq!(ExposureLevel::from_brightness(0.3), ExposureLevel::SlightlyDark);
        assert_eq!(ExposureLevel::from_brightness(0.5), ExposureLevel::WellExposed);
        assert_eq!(ExposureLevel::from_brightness(0.7), ExposureLevel::SlightlyBright);
        assert_eq!(ExposureLevel::from_brightness(0.9), ExposureLevel::Overexposed);
    }
    
    #[test]
    fn test_exposure_level_quality_score() {
        assert_eq!(ExposureLevel::WellExposed.quality_score(), 1.0);
        assert_eq!(ExposureLevel::SlightlyDark.quality_score(), 0.8);
        assert_eq!(ExposureLevel::SlightlyBright.quality_score(), 0.8);
        assert_eq!(ExposureLevel::Underexposed.quality_score(), 0.3);
        assert_eq!(ExposureLevel::Overexposed.quality_score(), 0.3);
    }
    
    #[test]
    fn test_exposure_analyzer_creation() {
        let analyzer = ExposureAnalyzer::default();
        assert_eq!(analyzer.dark_threshold, 30);
        assert_eq!(analyzer.bright_threshold, 225);
        
        let custom_analyzer = ExposureAnalyzer::new(20, 240);
        assert_eq!(custom_analyzer.dark_threshold, 20);
        assert_eq!(custom_analyzer.bright_threshold, 240);
    }
    
    #[test]
    fn test_rgb_to_luminance() {
        let analyzer = ExposureAnalyzer::default();
        let rgb_data = vec![255, 255, 255, 0, 0, 0]; // White, Black
        let luminance = analyzer.rgb_to_luminance(&rgb_data, 2, 1);
        
        assert_eq!(luminance.len(), 2);
        assert!(luminance[0] > 250); // White should be bright
        assert!(luminance[1] < 5);   // Black should be dark
    }
    
    #[test]
    fn test_histogram_calculation() {
        let analyzer = ExposureAnalyzer::default();
        let luminance = vec![0, 128, 255, 128]; // Various brightness levels
        let histogram = analyzer.calculate_histogram(&luminance);
        
        assert_eq!(histogram.len(), 256);
        assert_eq!(histogram[0], 1);   // One black pixel
        assert_eq!(histogram[128], 2); // Two mid-gray pixels
        assert_eq!(histogram[255], 1); // One white pixel
    }
    
    #[test]
    fn test_dark_frame_analysis() {
        let analyzer = ExposureAnalyzer::default();
        let dark_frame = create_test_frame_with_brightness(50, 50, 20);
        
        let metrics = analyzer.analyze_frame(&dark_frame);
        
        assert!(metrics.mean_brightness < 0.2);
        assert_eq!(metrics.exposure_level, ExposureLevel::Underexposed);
        assert!(metrics.dark_pixel_ratio > 0.5);
    }
    
    #[test]
    fn test_bright_frame_analysis() {
        let analyzer = ExposureAnalyzer::default();
        let bright_frame = create_test_frame_with_brightness(50, 50, 240);
        
        let metrics = analyzer.analyze_frame(&bright_frame);
        
        assert!(metrics.mean_brightness > 0.8);
        assert_eq!(metrics.exposure_level, ExposureLevel::Overexposed);
        assert!(metrics.bright_pixel_ratio > 0.5);
    }
    
    #[test]
    fn test_well_exposed_frame() {
        let analyzer = ExposureAnalyzer::default();
        let well_exposed_frame = create_test_frame_with_brightness(50, 50, 128);
        
        let metrics = analyzer.analyze_frame(&well_exposed_frame);
        
        assert!(metrics.mean_brightness > 0.4 && metrics.mean_brightness < 0.6);
        assert_eq!(metrics.exposure_level, ExposureLevel::WellExposed);
        assert!(analyzer.is_acceptable_exposure(&metrics));
    }
    
    #[test]
    fn test_exposure_correction() {
        let analyzer = ExposureAnalyzer::default();
        
        let dark_metrics = ExposureMetrics {
            mean_brightness: 0.1,
            brightness_std: 0.05,
            histogram: vec![0; 256],
            dark_pixel_ratio: 0.8,
            bright_pixel_ratio: 0.0,
            dynamic_range: 0.2,
            exposure_level: ExposureLevel::Underexposed,
            quality_score: 0.3,
        };
        
        match analyzer.get_exposure_correction(&dark_metrics) {
            ExposureCorrection::IncreaseExposure(factor) => {
                assert!(factor > 1.0);
            }
            _ => panic!("Expected IncreaseExposure for dark image"),
        }
    }
}