use crate::types::CameraFrame;
use crate::constants::{BLUR_VARIANCE_SHARP, BLUR_VARIANCE_GOOD, BLUR_VARIANCE_MODERATE, BLUR_VARIANCE_BLURRY, QUALITY_SCORE_SHARP, QUALITY_SCORE_GOOD, QUALITY_SCORE_MODERATE, QUALITY_SCORE_BLURRY, QUALITY_SCORE_VERY_BLURRY, DEFAULT_VARIANCE_THRESHOLD, DEFAULT_GRADIENT_THRESHOLD};
use serde::{Deserialize, Serialize};

/// Blur detection levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlurLevel {
    /// Very clear, minimal blur. Suitable for high quality capture.
    Sharp,
    /// Slight blur, still acceptable for general use.
    Good,
    /// Noticeable blur, borderline acceptable depending on use case.
    Moderate,
    /// Significant blur, poor quality. Should likely be discarded.
    Blurry,
    /// Severely blurred, unusable for any purpose.
    VeryBlurry,
}

impl BlurLevel {
    /// Convert blur variance to blur level
    #[must_use]
    pub fn from_variance(variance: f64) -> Self {
        if variance > BLUR_VARIANCE_SHARP {
            Self::Sharp
        } else if variance > BLUR_VARIANCE_GOOD {
            Self::Good
        } else if variance > BLUR_VARIANCE_MODERATE {
            Self::Moderate
        } else if variance > BLUR_VARIANCE_BLURRY {
            Self::Blurry
        } else {
            Self::VeryBlurry
        }
    }

    /// Get quality score (0.0 to 1.0)
    #[must_use]
    pub fn quality_score(self) -> f32 {
        match self {
            Self::Sharp => QUALITY_SCORE_SHARP,
            Self::Good => QUALITY_SCORE_GOOD,
            Self::Moderate => QUALITY_SCORE_MODERATE,
            Self::Blurry => QUALITY_SCORE_BLURRY,
            Self::VeryBlurry => QUALITY_SCORE_VERY_BLURRY,
        }
    }
}

/// Blur detection metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlurMetrics {
    /// Laplacian variance (higher = sharper).
    /// Typically used as the primary metric for focus detection.
    pub variance: f64,
    /// Sobel gradient magnitude.
    /// Measures the strength of edges in the image.
    pub gradient_magnitude: f64,
    /// Density of detected edges.
    /// Higher density usually correlates with more detail.
    pub edge_density: f64,
    /// Overall blur assessment level.
    pub blur_level: BlurLevel,
    /// Normalized quality score (0.0 to 1.0).
    pub quality_score: f32,
}

/// Blur detector using multiple algorithms.
///
/// Combines Laplacian variance, Sobel gradients, and edge density to determine
/// if an image is in focus.
pub struct BlurDetector {
    /// Threshold for variance-based detection
    threshold_variance: f64,
    /// Threshold for gradient-based detection
    threshold_gradient: f64,
}

impl Default for BlurDetector {
    fn default() -> Self {
        Self {
            threshold_variance: DEFAULT_VARIANCE_THRESHOLD, // Threshold for variance-based detection
            threshold_gradient: DEFAULT_GRADIENT_THRESHOLD,  // Threshold for gradient-based detection
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
        let grayscale = Self::rgb_to_grayscale(&frame.data, frame.width, frame.height);

        // Calculate Laplacian variance (primary blur metric)
        let variance = Self::calculate_laplacian_variance(&grayscale, frame.width, frame.height);

        // Calculate Sobel gradient magnitude
        let gradient_magnitude =
            Self::calculate_sobel_gradient(&grayscale, frame.width, frame.height);

        // Calculate edge density
        let edge_density = Self::calculate_edge_density(&grayscale, frame.width, frame.height);

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
    fn rgb_to_grayscale(rgb_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let mut grayscale = Vec::with_capacity((width * height) as usize);

        for i in (0..rgb_data.len()).step_by(3) {
            // Safety check for buffer overrun
            if i + 2 >= rgb_data.len() {
                break;
            }
            let r = f32::from(rgb_data[i]);
            let g = f32::from(rgb_data[i + 1]);
            let b = f32::from(rgb_data[i + 2]);

            // Standard luminance formula
            let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            grayscale.push(gray);
        }

        grayscale
    }

    /// Calculate Laplacian variance for blur detection
    fn calculate_laplacian_variance(grayscale: &[u8], width: u32, height: u32) -> f64 {
        let laplacian_kernel = [0, -1, 0, -1, 4, -1, 0, -1, 0];

        let mut laplacian_values = Vec::new();

        // Apply Laplacian filter
        // We skip the border pixels to avoid boundary checking for 3x3 kernel
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let mut sum = 0i32;

                for ky in 0..3 {
                    for kx in 0..3 {
                        // Safe: x/y are loop indices bounded by (1..width-1)/(1..height-1)
                        #[allow(clippy::cast_possible_wrap)]
                        let pixel_y = (y as i32 + ky - 1) as usize;
                        #[allow(clippy::cast_possible_wrap)]
                        let pixel_x = (x as i32 + kx - 1) as usize;
                        let pixel_index = pixel_y * width as usize + pixel_x;
                        
                        // Bounds check not strictly necessary due to loop limits but safe
                        if let Some(&val) = grayscale.get(pixel_index) {
                             let kernel_value = laplacian_kernel[(ky * 3 + kx) as usize];
                             sum += i32::from(val) * kernel_value;
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

        let count = laplacian_values.len() as f64;
        let mean = f64::from(laplacian_values.iter().sum::<i32>()) / count;
        
        laplacian_values
            .iter()
            .map(|&x| (f64::from(x) - mean).powi(2))
            .sum::<f64>()
            / count
    }

    /// Calculate Sobel gradient magnitude
    fn calculate_sobel_gradient(grayscale: &[u8], width: u32, height: u32) -> f64 {
        let sobel_x = [-1, 0, 1, -2, 0, 2, -1, 0, 1];
        let sobel_y = [-1, -2, -1, 0, 0, 0, 1, 2, 1];

        let mut gradients = Vec::new();

        // Skip borders
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let mut gx = 0i32;
                let mut gy = 0i32;

                for ky in 0..3 {
                    for kx in 0..3 {
                        // Safe: x/y are loop indices bounded by (1..width-1)/(1..height-1)
                        #[allow(clippy::cast_possible_wrap)]
                        let pixel_y = (y as i32 + ky - 1) as usize;
                        #[allow(clippy::cast_possible_wrap)]
                        let pixel_x = (x as i32 + kx - 1) as usize;
                        let pixel_index = pixel_y * width as usize + pixel_x;

                        if let Some(&val) = grayscale.get(pixel_index) {
                            let pixel_value = i32::from(val);
                            let kernel_idx = (ky * 3 + kx) as usize;
                            gx += pixel_value * sobel_x[kernel_idx];
                            gy += pixel_value * sobel_y[kernel_idx];
                        }
                    }
                }

                let magnitude = (f64::from(gx * gx + gy * gy)).sqrt();
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
    fn calculate_edge_density(grayscale: &[u8], width: u32, height: u32) -> f64 {
        let edge_threshold = 50;
        let mut edge_count = 0;
        let mut total_pixels = 0;

        // Using simple neighbor difference
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let center_idx = (y * width + x) as usize;
                
                let center = match grayscale.get(center_idx) {
                    Some(&c) => i32::from(c),
                    None => continue,
                };

                // Check 8-connected neighbors
                // Pre-calculating indices could be optimized but this is readable
                let neighbors_indices = [
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
                for &neighbor_idx in &neighbors_indices {
                    if let Some(&val) = grayscale.get(neighbor_idx) {
                        let diff = (center - i32::from(val)).abs();
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
            f64::from(edge_count) / f64::from(total_pixels)
        } else {
            0.0
        }
    }

    /// Check if frame meets minimum quality threshold
    pub fn is_acceptable_quality(&self, metrics: &BlurMetrics) -> bool {
        metrics.variance > self.threshold_variance
            && metrics.gradient_magnitude > self.threshold_gradient
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
            data[i] = 128; // R
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
        let epsilon = 1e-10;
        assert!((BlurLevel::Sharp.quality_score() - 1.0).abs() < epsilon);
        assert!((BlurLevel::Good.quality_score() - 0.8).abs() < epsilon);
        assert!((BlurLevel::Moderate.quality_score() - 0.6).abs() < epsilon);
        assert!((BlurLevel::Blurry.quality_score() - 0.3).abs() < epsilon);
        assert!((BlurLevel::VeryBlurry.quality_score() - 0.1).abs() < epsilon);
    }

    #[test]
    fn test_blur_detector_creation() {
        let epsilon = 1e-10;
        let detector = BlurDetector::default();
        assert!((detector.threshold_variance - 200.0).abs() < epsilon);
        assert!((detector.threshold_gradient - 50.0).abs() < epsilon);

        let custom_detector = BlurDetector::new(300.0, 60.0);
        assert!((custom_detector.threshold_variance - 300.0).abs() < epsilon);
        assert!((custom_detector.threshold_gradient - 60.0).abs() < epsilon);
    }

    #[test]
    fn test_rgb_to_grayscale() {
        let rgb_data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255]; // Red, Green, Blue
        let grayscale = BlurDetector::rgb_to_grayscale(&rgb_data, 3, 1);

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
