/// Auto-capture quality validation module
///
/// Provides automated quality assessment for captured frames including
/// blur detection, exposure analysis, and overall image quality scoring.
pub mod blur;
/// Exposure analysis and correction recommendations.
pub mod exposure;
/// Quality validation summary and reporting.
pub mod validator;

pub use blur::{BlurDetector, BlurLevel, BlurMetrics};
pub use exposure::{ExposureAnalyzer, ExposureLevel, ExposureMetrics};
pub use validator::{QualityReport, QualityScore, QualityValidator, ValidationConfig};

/// Smart capture triggering based on quality metrics.
pub mod smart_trigger;
pub use smart_trigger::{SmartTrigger, TriggerConfig, TriggerStatus};
