//! ContextLite integration for plant photography analysis
//!
//! Provides AI-powered analysis of plant photographs using ContextLite
//! to offer cultivation insights and growth recommendations.

use crate::errors::CameraError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[cfg(feature = "contextlite")]
use contextlite_client::ContextLiteClient;

/// Photo metadata for ContextLite analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoMetadata {
    pub id: Uuid,
    pub camera_name: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
    pub timestamp: DateTime<Utc>,
    pub camera_settings: Option<CameraSettings>,
}

/// Camera settings for photo capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    pub exposure: Option<ExposureSettings>,
    pub white_balance: Option<String>,
    pub focus_mode: Option<String>,
}

/// Exposure settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureSettings {
    pub iso: Option<u32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<f32>,
}

/// ContextLite integration for plant photography analysis
#[derive(Debug, Clone)]
pub struct PlantPhotoAnalyzer {
    #[cfg(feature = "contextlite")]
    #[allow(dead_code)]
    client: ContextLiteClient,
    #[allow(dead_code)]
    workspace_id: String,
}

/// Photo analysis query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoAnalysisQuery {
    pub photo_id: Uuid,
    pub query: String,
    pub include_growth_analysis: bool,
    pub include_health_analysis: bool,
    pub max_documents: usize,
    pub max_tokens: usize,
}

/// Photo analysis response with AI insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoAnalysisResponse {
    pub photo_id: Uuid,
    pub query: String,
    pub analysis: String,
    pub recommendations: Vec<String>,
    pub growth_indicators: Vec<GrowthIndicator>,
    pub health_indicators: Vec<HealthIndicator>,
    pub confidence_score: f32,
}

/// Growth indicator identified in photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthIndicator {
    pub indicator_type: String,
    pub description: String,
    pub confidence: f32,
    pub location: Option<PhotoLocation>,
}

/// Health indicator identified in photo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    pub indicator_type: String,
    pub description: String,
    pub severity: HealthSeverity,
    pub confidence: f32,
    pub location: Option<PhotoLocation>,
}

/// Location within photo for annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoLocation {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Health indicator severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSeverity {
    Excellent,
    Good,
    Warning,
    Critical,
}

impl PlantPhotoAnalyzer {
    /// Create new plant photo analyzer
    #[cfg(feature = "contextlite")]
    pub fn new(base_url: &str, _auth_token: &str, workspace_id: &str) -> Result<Self, CameraError> {
        let client = ContextLiteClient::new(base_url)
            .map_err(|e| CameraError::InitializationError(format!("ContextLite client error: {}", e)))?;
        
        Ok(Self {
            client,
            workspace_id: workspace_id.to_string(),
        })
    }

    /// Create new plant photo analyzer (no-op without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub fn new(_base_url: &str, _auth_token: &str, workspace_id: &str) -> Result<Self, CameraError> {
        Ok(Self {
            workspace_id: workspace_id.to_string(),
        })
    }

    /// Analyze plant photo for growth and health insights
    #[cfg(feature = "contextlite")]
    pub async fn analyze_plant_photo(
        &self,
        photo_metadata: &PhotoMetadata,
        photo_path: &str,
        query: &str,
    ) -> Result<PhotoAnalysisResponse, CameraError> {
        // Build context from photo metadata
        let mut context_parts = vec![
            format!("Photo ID: {}", photo_metadata.id),
            format!("Camera: {}", photo_metadata.camera_name),
            format!("Resolution: {}x{}", photo_metadata.width, photo_metadata.height),
            format!("File: {}", photo_path),
        ];

        if let Some(camera_settings) = &photo_metadata.camera_settings {
            if let Some(exposure) = &camera_settings.exposure {
                if let Some(iso) = exposure.iso {
                    context_parts.push(format!("ISO: {}", iso));
                }
                if let Some(aperture) = exposure.aperture {
                    context_parts.push(format!("Aperture: f/{}", aperture));
                }
                if let Some(shutter) = exposure.shutter_speed {
                    context_parts.push(format!("Shutter: {}s", shutter));
                }
            }
        }

        // TODO: Implement actual ContextLite photo analysis API call
        // For now, provide mock analysis based on query
        let analysis = format!(
            "Photo analysis for {} using camera {} at {}x{} resolution",
            query,
            photo_metadata.camera_name,
            photo_metadata.width,
            photo_metadata.height
        );

        let recommendations = extract_photo_recommendations(&analysis);
        let growth_indicators = extract_growth_indicators(&analysis);
        let health_indicators = extract_health_indicators(&analysis);

        Ok(PhotoAnalysisResponse {
            photo_id: photo_metadata.id,
            query: query.to_string(),
            analysis,
            recommendations,
            growth_indicators,
            health_indicators,
            confidence_score: 0.75,
        })
    }

    /// Analyze plant photo for growth and health insights (mock without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn analyze_plant_photo(
        &self,
        photo_metadata: &PhotoMetadata,
        _photo_path: &str,
        query: &str,
    ) -> Result<PhotoAnalysisResponse, CameraError> {
        // Mock response when ContextLite is not available
        Ok(PhotoAnalysisResponse {
            photo_id: photo_metadata.id,
            query: query.to_string(),
            analysis: "ContextLite feature not enabled".to_string(),
            recommendations: vec!["Enable ContextLite feature for AI photo analysis".to_string()],
            growth_indicators: vec![],
            health_indicators: vec![],
            confidence_score: 0.0,
        })
    }

    /// Get cultivation advice based on photo analysis
    #[cfg(feature = "contextlite")]
    pub async fn get_cultivation_advice(&self, photo_analysis: &PhotoAnalysisResponse) -> Result<String, CameraError> {
        // TODO: Implement actual ContextLite cultivation advice API call
        Ok(format!("Mock cultivation advice based on photo analysis: {}", photo_analysis.analysis))
    }

    /// Get cultivation advice based on photo analysis (mock without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn get_cultivation_advice(&self, photo_analysis: &PhotoAnalysisResponse) -> Result<String, CameraError> {
        Ok(format!("ContextLite feature not enabled for cultivation advice: {}", photo_analysis.query))
    }

    /// Index photo analysis data to ContextLite knowledge base
    #[cfg(feature = "contextlite")]
    pub async fn index_photo_analysis(
        &self,
        photo_metadata: &PhotoMetadata,
        analysis: &PhotoAnalysisResponse,
    ) -> Result<(), CameraError> {
        // TODO: Implement actual ContextLite document indexing
        log::info!(
            "Would index photo analysis: {} for photo {}",
            analysis.query,
            photo_metadata.id
        );
        Ok(())
    }

    /// Index photo analysis data (no-op without contextlite feature)
    #[cfg(not(feature = "contextlite"))]
    pub async fn index_photo_analysis(
        &self,
        _photo_metadata: &PhotoMetadata,
        _analysis: &PhotoAnalysisResponse,
    ) -> Result<(), CameraError> {
        Ok(()) // No-op when ContextLite is not available
    }
}

/// Extract photo-based recommendations
fn extract_photo_recommendations(analysis: &str) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if analysis.contains("lighting") || analysis.contains("light") {
        recommendations.push("Consider adjusting grow light intensity or duration".to_string());
    }
    
    if analysis.contains("deficiency") || analysis.contains("yellowing") {
        recommendations.push("Check nutrient levels and feeding schedule".to_string());
    }
    
    if analysis.contains("pest") || analysis.contains("damage") {
        recommendations.push("Inspect for pests and consider treatment options".to_string());
    }
    
    if analysis.contains("harvest") || analysis.contains("trichome") {
        recommendations.push("Consider harvest timing evaluation".to_string());
    }
    
    if recommendations.is_empty() {
        recommendations.push("Continue monitoring plant health and growth".to_string());
    }
    
    recommendations
}

/// Extract growth indicators from analysis
fn extract_growth_indicators(analysis: &str) -> Vec<GrowthIndicator> {
    let mut indicators = Vec::new();
    
    if analysis.contains("flower") || analysis.contains("bud") {
        indicators.push(GrowthIndicator {
            indicator_type: "flowering".to_string(),
            description: "Flowering development detected".to_string(),
            confidence: 0.8,
            location: None,
        });
    }
    
    if analysis.contains("growth") || analysis.contains("height") {
        indicators.push(GrowthIndicator {
            indicator_type: "vegetative_growth".to_string(),
            description: "Active vegetative growth observed".to_string(),
            confidence: 0.7,
            location: None,
        });
    }
    
    indicators
}

/// Extract health indicators from analysis
fn extract_health_indicators(analysis: &str) -> Vec<HealthIndicator> {
    let mut indicators = Vec::new();
    
    if analysis.contains("healthy") || analysis.contains("good") {
        indicators.push(HealthIndicator {
            indicator_type: "overall_health".to_string(),
            description: "Plant appears healthy".to_string(),
            severity: HealthSeverity::Good,
            confidence: 0.8,
            location: None,
        });
    }
    
    if analysis.contains("yellowing") || analysis.contains("deficiency") {
        indicators.push(HealthIndicator {
            indicator_type: "nutrient_deficiency".to_string(),
            description: "Possible nutrient deficiency detected".to_string(),
            severity: HealthSeverity::Warning,
            confidence: 0.7,
            location: None,
        });
    }
    
    if analysis.contains("pest") || analysis.contains("damage") {
        indicators.push(HealthIndicator {
            indicator_type: "pest_damage".to_string(),
            description: "Potential pest damage observed".to_string(),
            severity: HealthSeverity::Critical,
            confidence: 0.6,
            location: None,
        });
    }
    
    indicators
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{ExposureSettings, CameraSettings};
    use chrono::Utc;

    #[tokio::test]
    async fn test_plant_photo_analyzer_creation() {
        let analyzer = PlantPhotoAnalyzer::new(
            "http://localhost:8090",
            "test-token",
            "budsy-cultivation"
        ).expect("Failed to create analyzer");

        assert_eq!(analyzer.workspace_id, "budsy-cultivation");
    }

    #[tokio::test]
    async fn test_mock_photo_analysis() {
        let analyzer = PlantPhotoAnalyzer::new(
            "http://localhost:8090",
            "test-token",
            "test-workspace"
        ).expect("Failed to create analyzer");

        let photo_metadata = PhotoMetadata {
            id: Uuid::new_v4(),
            camera_name: "Test Camera".to_string(),
            width: 1920,
            height: 1080,
            format: "JPEG".to_string(),
            file_size: 2048576,
            timestamp: Utc::now(),
            camera_settings: Some(CameraSettings {
                exposure: Some(ExposureSettings {
                    iso: Some(800),
                    aperture: Some(2.8),
                    shutter_speed: Some(1.0/60.0),
                }),
                white_balance: Some("auto".to_string()),
                focus_mode: Some("macro".to_string()),
            }),
        };

        let response = analyzer.analyze_plant_photo(
            &photo_metadata,
            "/test/photo.jpg",
            "How is my plant growing?"
        ).await.expect("Failed to get photo analysis");

        assert_eq!(response.photo_id, photo_metadata.id);
        assert!(!response.recommendations.is_empty());
    }

    #[test]
    fn test_photo_analysis_extraction() {
        let test_analysis = "The plant shows healthy flowering development with good lighting conditions";
        
        let recommendations = extract_photo_recommendations(&test_analysis);
        let growth_indicators = extract_growth_indicators(&test_analysis);
        let health_indicators = extract_health_indicators(&test_analysis);
        
        assert!(!recommendations.is_empty());
        assert!(!growth_indicators.is_empty());
        assert!(!health_indicators.is_empty());
    }
}