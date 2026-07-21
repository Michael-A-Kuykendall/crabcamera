//! System Registry & Feature Manifest
//!
//! This module serves as the authoritative source of truth for all capabilities,
//! commands, and architectural pathways in the CrabCamera system.
//!
//! # Core Philosophy
//! This registry acts as a compile-time and runtime map of "What Works" vs "What is Planned".
//! If a feature is listed here with status `Implemented`, it MUST work.
//! If a feature is listed as `Stub` or `Planned`, it is explicitly known to be incomplete.
//!
//! This prevents "Ghost Features" (vaporware) by forcing every feature to be registered
//! with its implementation status.

use serde::{Deserialize, Serialize};

/// Status of a system capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureStatus {
    /// Fully implemented, tested, and ready for production
    Implemented,
    /// Implemented but requires more extensive testing
    Beta,
    /// Placeholder structure exists, but logic is incomplete (e.g. returns mock data)
    Stub,
    /// Not started, planned for future release
    Planned,
    /// Deprecated, will be removed
    Deprecated,
}

/// Category of system capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureCategory {
    /// Core camera operations (Open, Stream, Capture)
    Core,
    /// Advanced controls (Focus, Exposure, White Balance)
    Controls,
    /// Processing pipeline (Quality, Filters, Analysis)
    Processing,
    /// Recording and Encoding
    Recording,
    /// Integration (Tauri, Events, Permissions)
    Integration,
}

/// Metadata for a system feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureManifest {
    /// Unique identifier for the feature
    pub id: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// Current implementation status
    pub status: FeatureStatus,
    /// Feature category
    pub category: FeatureCategory,
    /// Where the implementation lives (module path)
    pub path: &'static str,
    /// Description of the feature
    pub description: &'static str,
}

/// The Global System Registry
pub struct SystemRegistry;

impl SystemRegistry {
    /// Get the complete manifest of system features
    #[must_use]
    pub fn get_manifest() -> Vec<FeatureManifest> {
        let mut features = Self::capture_and_control_features();
        features.extend(Self::quality_and_platform_features());
        features
    }

    /// Capture and advanced-control feature descriptors.
    fn capture_and_control_features() -> Vec<FeatureManifest> {
        vec![
            // Core Capture
            FeatureManifest {
                id: "capture.single",
                name: "Single Photo Capture",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Core,
                path: "src/commands/capture.rs",
                description: "Capture a single frame from a camera source",
            },
            FeatureManifest {
                id: "capture.sequence",
                name: "Burst Sequence Capture",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Core,
                path: "src/commands/capture.rs",
                description: "Capture multiple frames in rapid succession",
            },
            FeatureManifest {
                id: "capture.preview",
                name: "Live Preview Stream",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Core,
                path: "src/commands/capture.rs",
                description: "Continuous frame streaming for UI preview",
            },
            FeatureManifest {
                id: "capture.consolidated",
                name: "Consolidated Capture",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Core,
                path: "src/commands/capture.rs",
                description:
                    "Unified capture command routing to single/sequence/quality-retry modes",
            },
            // Advanced Controls
            FeatureManifest {
                id: "controls.focus",
                name: "Manual Focus Control",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Controls,
                path: "src/platform/mod.rs",
                description: "Manual control of lens focus distance (Platform dependent)",
            },
            FeatureManifest {
                id: "controls.exposure",
                name: "Exposure Control",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Controls,
                path: "src/platform/mod.rs",
                description: "Manual control of exposure time and ISO",
            },
            FeatureManifest {
                id: "controls.batch",
                name: "Batch Camera Settings",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Controls,
                path: "src/commands/advanced.rs",
                description:
                    "Apply multiple camera settings (focus/exposure/ISO/WB) in a single call",
            },
        ]
    }

    /// Quality-engine and platform-driver feature descriptors.
    fn quality_and_platform_features() -> Vec<FeatureManifest> {
        vec![
            // Quality Engine
            FeatureManifest {
                id: "quality.blur",
                name: "Blur Analysis",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Processing,
                path: "src/quality/blur.rs",
                description: "Laplacian variance analysis for sharpness detection",
            },
            FeatureManifest {
                id: "quality.exposure",
                name: "Exposure Analysis",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Processing,
                path: "src/quality/exposure.rs",
                description: "Histogram analysis for over/under-exposure detection",
            },
            // Platform Drivers
            FeatureManifest {
                id: "platform.windows",
                name: "Windows MediaFoundation Driver",
                status: FeatureStatus::Implemented,
                category: FeatureCategory::Core,
                path: "src/platform/windows/mod.rs",
                description: "Native Windows camera implementation",
            },
            FeatureManifest {
                id: "platform.macos",
                name: "MacOS AVFoundation Driver",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Core,
                path: "src/platform/macos.rs",
                description: "Native MacOS camera implementation (Basic controls)",
            },
            FeatureManifest {
                id: "platform.linux",
                name: "Linux V4L2 Driver",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Core,
                path: "src/platform/linux.rs",
                description: "Native Linux V4L2 camera implementation",
            },
            FeatureManifest {
                id: "platform.linux.formats",
                name: "Linux V4L2 Format Enumeration",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Core,
                path: "src/platform/linux.rs",
                description:
                    "Real format enumeration via dev.enum_formats() with hardcoded fallback",
            },
            FeatureManifest {
                id: "platform.windows.availability",
                name: "Windows Camera Availability Check",
                status: FeatureStatus::Beta,
                category: FeatureCategory::Core,
                path: "src/platform/windows/mod.rs",
                description:
                    "Availability based on is_stream_open(); deeper hardware probe deferred",
            },
        ]
    }

    /// Verify that all registered features point to valid code paths
    /// (This acts as a compile-time check if we link symbols directly)
    pub fn verify_linkage() {
        // This function doesn't run logic, but referencing symbols ensures they exist
        // at compile time. If a function is deleted, this registry will fail to compile.

        use crate::commands;

        // Linking Core Commands
        let _ = commands::capture::capture_single_photo;
        let _ = commands::capture::capture_photo_sequence;
        let _ = commands::capture::capture;
        let _ = commands::capture::start_camera_preview;
        let _ = commands::capture::stop_camera_preview;

        // Linking Advanced Commands
        let _ = commands::advanced::apply_camera_settings;

        // Linking Quality
        let _ = crate::quality::BlurDetector::new;
        let _ = crate::quality::ExposureAnalyzer::new;

        // Linking Platform (Generic)
        let _ = crate::platform::get_or_create_camera;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_integrity() {
        // Ensure manifest generates without error
        let manifest = SystemRegistry::get_manifest();
        assert!(!manifest.is_empty());

        // Verify linkage (compile-time check wrapped in test)
        SystemRegistry::verify_linkage();
    }

    #[test]
    fn test_no_stubs_in_production() {
        let manifest = SystemRegistry::get_manifest();
        for feature in manifest {
            assert_ne!(
                feature.status,
                FeatureStatus::Stub,
                "Feature '{}' is marked as Stub — either implement it or remove it from the manifest",
                feature.name
            );
        }
    }
}
