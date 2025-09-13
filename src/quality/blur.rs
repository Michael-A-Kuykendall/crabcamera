use crate::types::CameraFrame;
use serde::{Deserialize, Serialize};

/// Blur detection levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurLevel {
    Sharp,      // Very clear, minimal blur
    Good,       // Slight blur, still acceptable
    Moderate,   // Noticeable blur, borderline acceptable
    Blurry,     // Significant blur, poor quality
    VeryBlurry, // Severely blurred, unusable
}

impl BlurLevel {
    /// Convert blur variance to blur level
    pub fn from_variance(variance: f64) -> Self {
        if variance > 1000.0 {
            BlurLevel::Sharp
        } else if variance > 500.0 {
            BlurLevel::Good
        } else if variance > 200.0 {
            BlurLevel::Moderate
        } else if variance > 50.0 {
            BlurLevel::Blurry
        } else {
            BlurLevel::VeryBlurry
        }
    }
    
    /// Get quality score (0.0 to 1.0)
    pub fn quality_score(&self) -> f32 {
        match self {
            BlurLevel::Sharp => 1.0,
            BlurLevel::Good => 0.8,
            BlurLevel::Moderate => 0.6,
            BlurLevel::Blurry => 0.3,
            BlurLevel::VeryBlurry => 0.1,
        }
    }
}

/// Blur detection metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlurMetrics {
    pub variance: f64,           // Laplacian variance (higher = sharper)
    pub gradient_magnitude: f64, // Sobel gradient magnitude
    pub edge_density: f64,       // Density of detected edges
    pub blur_level: BlurLevel,   // Overall blur assessment
    pub quality_score: f32,      // Quality score (0.0 to 1.0)
}

/// Blur detector using multiple algorithms
pub struct BlurDetector {
    threshold_variance: f64,
    threshold_gradient: f64,
}

impl Default for BlurDetector {
    fn default() -> Self {
        Self {
            threshold_variance: 200.0,  // Threshold for variance-based detection
            threshold_gradient: 50.0,   // Threshold for gradient-based detection
        }
    }
}

impl BlurDetector {
    /// Create new blur detector with custom thresholds
    pub fn new(threshold_variance: f64, threshold_gradient: f64) -> Self {
        Self {
            threshold_variance,
            threshold_gradient,
        }
    }
    
    /// Analyze frame for blur
    pub fn analyze_frame(&self, frame: &CameraFrame) -> BlurMetrics {
        // Convert to grayscale for analysis
        let grayscale = self.rgb_to_grayscale(&frame.data, frame.width, frame.height);
        
        // Calculate Laplacian variance (primary blur metric)
        let variance = self.calculate_laplacian_variance(&grayscale, frame.width, frame.height);
        
        // Calculate Sobel gradient magnitude
        let gradient_magnitude = self.calculate_sobel_gradient(&grayscale, frame.width, frame.height);
        
        // Calculate edge density
        let edge_density = self.calculate_edge_density(&grayscale, frame.width, frame.height);
        
        // Determine blur level
        let blur_level = BlurLevel::from_variance(variance);
        let quality_score = blur_level.quality_score();
        
        BlurMetrics {
            variance,
            gradient_magnitude,
            edge_density,
            blur_level,
            quality_score,
        }
    }
    
    /// Convert RGB to grayscale
    fn rgb_to_grayscale(&self, rgb_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let mut grayscale = Vec::with_capacity((width * height) as usize);
        
        for i in (0..rgb_data.len()).step_by(3) {
            let r = rgb_data[i] as f32;
            let g = rgb_data[i + 1] as f32;
            let b = rgb_data[i + 2] as f32;
            
            // Standard luminance formula
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            grayscale.push(gray);
        }
        
        grayscale
    }
    
    /// Calculate Laplacian variance for blur detection
    fn calculate_laplacian_variance(&self, grayscale: &[u8], width: u32, height: u32) -> f64 {
        let laplacian_kernel = [
            0, -1,  0,
           -1,  4, -1,
            0, -1,  0
        ];
        
        let mut laplacian_values = Vec::new();
        
        // Apply Laplacian filter
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let mut sum = 0i32;
                
                for ky in 0..3 {
                    for kx in 0..3 {
                        let pixel_y = (y as i32 + ky - 1) as usize;
                        let pixel_x = (x as i32 + kx - 1) as usize;
                        let pixel_index = pixel_y * width as usize + pixel_x;
                        
                        if pixel_index < grayscale.len() {
                            let kernel_value = laplacian_kernel[(ky * 3 + kx) as usize];
                            sum += grayscale[pixel_index] as i32 * kernel_value;
                        }
                    }
                }
                
                laplacian_values.push(sum);
            }
        }
        
        // Calculate variance of Laplacian values
        if laplacian_values.is_empty() {
            return 0.0;
        }
        
        let mean = laplacian_values.iter().sum::<i32>() as f64 / laplacian_values.len() as f64;
        let variance = laplacian_values.iter()
            .map(|&x| (x as f64 - mean).powi(2))
            .sum::<f64>() / laplacian_values.len() as f64;
        
        variance
    }
    
    /// Calculate Sobel gradient magnitude
    fn calculate_sobel_gradient(&self, grayscale: &[u8], width: u32, height: u32) -> f64 {
        let sobel_x = [
            -1, 0, 1,
            -2, 0, 2,
            -1, 0, 1
        ];
        
        let sobel_y = [
            -1, -2, -1,
             0,  0,  0,
             1,  2,  1
        ];
        
        let mut gradients = Vec::new();
        
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let mut gx = 0i32;
                let mut gy = 0i32;
                
                for ky in 0..3 {
                    for kx in 0..3 {
                        let pixel_y = (y as i32 + ky - 1) as usize;
                        let pixel_x = (x as i32 + kx - 1) as usize;
                        let pixel_index = pixel_y * width as usize + pixel_x;
                        
                        if pixel_index < grayscale.len() {
                            let pixel_value = grayscale[pixel_index] as i32;
                            gx += pixel_value * sobel_x[(ky * 3 + kx) as usize];
                            gy += pixel_value * sobel_y[(ky * 3 + kx) as usize];
                        }
                    }
                }
                
                let magnitude = ((gx * gx + gy * gy) as f64).sqrt();
                gradients.push(magnitude);
            }
        }
        
        if gradients.is_empty() {
            0.0
        } else {
            gradients.iter().sum::<f64>() / gradients.len() as f64
        }
    }
    
    /// Calculate edge density using simple threshold
    fn calculate_edge_density(&self, grayscale: &[u8], width: u32, height: u32) -> f64 {
        let edge_threshold = 50;
        let mut edge_count = 0;
        let mut total_pixels = 0;
        
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let center_idx = (y * width + x) as usize;
                if center_idx >= grayscale.len() {
                    continue;
                }
                
                let center = grayscale[center_idx] as i32;
                
                // Check 8-connected neighbors
                let neighbors = [
                    ((y - 1) * width + (x - 1)) as usize,
                    ((y - 1) * width + x) as usize,
                    ((y - 1) * width + (x + 1)) as usize,
                    (y * width + (x - 1)) as usize,
                    (y * width + (x + 1)) as usize,
                    ((y + 1) * width + (x - 1)) as usize,
                    ((y + 1) * width + x) as usize,
                    ((y + 1) * width + (x + 1)) as usize,
                ];
                
                let mut max_diff = 0;
                for &neighbor_idx in &neighbors {
                    if neighbor_idx < grayscale.len() {
                        let diff = (center - grayscale[neighbor_idx] as i32).abs();
                        max_diff = max_diff.max(diff);
                    }
                }
                
                if max_diff > edge_threshold {
                    edge_count += 1;
                }
                total_pixels += 1;
            }
        }
        
        if total_pixels > 0 {
            edge_count as f64 / total_pixels as f64
        } else {
            0.0
        }
    }
    
    /// Check if frame meets minimum quality threshold
    pub fn is_acceptable_quality(&self, metrics: &BlurMetrics) -> bool {
        metrics.variance > self.threshold_variance && 
        metrics.gradient_magnitude > self.threshold_gradient
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_frame(width: u32, height: u32) -> CameraFrame {
        let size = (width * height * 3) as usize;
        let mut data = vec![0u8; size];
        
        // Create a simple pattern for testing
        for i in (0..size).step_by(3) {
            data[i] = 128;     // R
            data[i + 1] = 128; // G  
            data[i + 2] = 128; // B
        }
        
        CameraFrame::new(data, width, height, "test".to_string())
    }
    
    #[test]
    fn test_blur_level_from_variance() {
        assert_eq!(BlurLevel::from_variance(1500.0), BlurLevel::Sharp);
        assert_eq!(BlurLevel::from_variance(800.0), BlurLevel::Good);
        assert_eq!(BlurLevel::from_variance(300.0), BlurLevel::Moderate);
        assert_eq!(BlurLevel::from_variance(100.0), BlurLevel::Blurry);
        assert_eq!(BlurLevel::from_variance(10.0), BlurLevel::VeryBlurry);
    }
    
    #[test]
    fn test_blur_level_quality_score() {
        assert_eq!(BlurLevel::Sharp.quality_score(), 1.0);
        assert_eq!(BlurLevel::Good.quality_score(), 0.8);
        assert_eq!(BlurLevel::Moderate.quality_score(), 0.6);
        assert_eq!(BlurLevel::Blurry.quality_score(), 0.3);
        assert_eq!(BlurLevel::VeryBlurry.quality_score(), 0.1);
    }
    
    #[test]
    fn test_blur_detector_creation() {
        let detector = BlurDetector::default();
        assert_eq!(detector.threshold_variance, 200.0);
        assert_eq!(detector.threshold_gradient, 50.0);
        
        let custom_detector = BlurDetector::new(300.0, 60.0);
        assert_eq!(custom_detector.threshold_variance, 300.0);
        assert_eq!(custom_detector.threshold_gradient, 60.0);
    }
    
    #[test]
    fn test_rgb_to_grayscale() {
        let detector = BlurDetector::default();
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // Red, Green, Blue
        let grayscale = detector.rgb_to_grayscale(&rgb_data, 3, 1);
        
        assert_eq!(grayscale.len(), 3);
        // Check luminance conversion is working (approximate values)
        assert!(grayscale[0] > 70 && grayscale[0] < 80); // Red
        assert!(grayscale[1] > 140 && grayscale[1] < 150); // Green
        assert!(grayscale[2] > 25 && grayscale[2] < 35); // Blue
    }
    
    #[test]
    fn test_frame_analysis() {
        let detector = BlurDetector::default();
        let frame = create_test_frame(100, 100);
        
        let metrics = detector.analyze_frame(&frame);
        
        assert!(metrics.variance >= 0.0);
        assert!(metrics.gradient_magnitude >= 0.0);
        assert!(metrics.edge_density >= 0.0 && metrics.edge_density <= 1.0);
        assert!(metrics.quality_score >= 0.0 && metrics.quality_score <= 1.0);
    }
    
    #[test]
    fn test_quality_threshold() {
        let detector = BlurDetector::new(100.0, 30.0);
        
        let good_metrics = BlurMetrics {
            variance: 150.0,
            gradient_magnitude: 40.0,
            edge_density: 0.3,
            blur_level: BlurLevel::Good,
            quality_score: 0.8,
        };
        
        let bad_metrics = BlurMetrics {
            variance: 50.0,
            gradient_magnitude: 20.0,
            edge_density: 0.1,
            blur_level: BlurLevel::Blurry,
            quality_score: 0.3,
        };
        
        assert!(detector.is_acceptable_quality(&good_metrics));
        assert!(!detector.is_acceptable_quality(&bad_metrics));
    }
}