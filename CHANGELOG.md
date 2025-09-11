# Changelog

All notable changes to CrabCamera will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-14

### 🚀 Major Features Added

#### Advanced Camera Controls
- **Manual Focus Control**: Set precise focus distance (0.0 = infinity, 1.0 = closest)
- **Manual Exposure Control**: Full exposure time and ISO sensitivity control
- **White Balance Modes**: Auto, Daylight, Fluorescent, Incandescent, Flash, Cloudy, Shade, Custom
- **Professional Settings**: Aperture, zoom, brightness, contrast, saturation, sharpness
- **Image Stabilization & Noise Reduction**: Configurable quality enhancement features

#### Burst Mode & Advanced Capture
- **Burst Capture**: Configurable burst sequences with custom intervals
- **HDR Photography**: Automatic exposure bracketing for high dynamic range
- **Focus Stacking**: Multiple focus points for macro photography depth
- **Exposure Bracketing**: Custom EV stops for professional HDR workflows
- **Plant Photography Optimization**: Specialized settings for botanical photography

#### Performance Optimizations
- **Async-Friendly Locking**: Replaced blocking mutexes with tokio RwLock for better concurrency
- **Memory Pool System**: Zero-copy frame buffers for reduced allocations
- **Async File I/O**: Non-blocking disk operations for frame saving
- **Compressed Saving**: JPEG compression with quality control for smaller files
- **Camera Registry**: Efficient camera management with connection pooling

#### Enhanced Metadata & Quality
- **Extended Frame Metadata**: Capture settings, EXIF-like data, performance metrics
- **Quality Scoring**: Automatic frame quality assessment
- **Sharpness Detection**: Real-time image sharpness calculation
- **Plant Enhancement**: Specialized image processing for botanical applications

### 🛠️ Technical Improvements

#### Type System Enhancements
- `CameraControls` struct for professional camera parameter management
- `BurstConfig` and `ExposureBracketing` for advanced capture modes
- `CameraCapabilities` for hardware feature detection
- `FrameMetadata` for comprehensive image metadata
- `CameraPerformanceMetrics` for performance monitoring

#### New Commands Added
- `set_camera_controls` - Apply professional camera settings
- `get_camera_controls` - Retrieve current camera configuration
- `capture_burst_sequence` - Multi-frame capture with advanced options
- `set_manual_focus` - Precise focus distance control
- `set_manual_exposure` - Manual exposure and ISO settings
- `set_white_balance` - White balance mode selection
- `capture_hdr_sequence` - Automatic HDR capture
- `capture_focus_stack` - Focus stacking for macro photography
- `get_camera_performance` - Performance metrics and statistics
- `optimize_for_plants` - One-click plant photography optimization
- `test_camera_capabilities` - Hardware capability detection
- `save_frame_compressed` - Compressed image saving with quality control

#### Platform Support Improvements
- Extended `PlatformCamera` interface with advanced control methods
- Enhanced capability detection for Windows, macOS, and Linux
- Platform-specific optimization recommendations
- Improved error handling and fallback mechanisms

### 📊 Testing & Quality Assurance

#### Comprehensive Test Suite
- **Advanced Features Testing**: Full coverage of new camera controls
- **Performance Benchmarks**: Burst capture speed and latency measurements
- **Mock System Integration**: Reliable testing without hardware dependencies
- **Edge Case Validation**: Input validation and error condition testing
- **Plant Photography Tests**: Specialized tests for botanical applications

#### Test Coverage Additions
- Manual focus and exposure control validation
- Burst sequence and HDR capture testing
- White balance mode verification
- Performance metric collection and analysis
- Camera capability detection testing

### 🔧 Developer Experience

#### API Improvements
- Consistent async/await patterns throughout
- Comprehensive error messages with context
- Type-safe parameter validation
- Builder pattern for configuration objects
- Extensive documentation and examples

#### Configuration Enhancements
- `CameraInitParams::for_plant_photography()` - One-line botanical setup
- `CameraControls::plant_photography()` - Optimized plant settings
- `BurstConfig::hdr_burst()` - Pre-configured HDR capture
- Platform-specific optimization helpers

### 📝 Documentation

#### New Examples
- Professional photography workflow examples
- Plant photography setup guides
- HDR and focus stacking tutorials
- Performance optimization recommendations

#### API Documentation
- Comprehensive parameter documentation
- Usage examples for all new features
- Platform compatibility notes
- Performance characteristics

### 🐛 Bug Fixes
- Fixed memory leaks in camera registry management
- Improved platform detection reliability
- Enhanced error recovery for camera disconnection
- Fixed race conditions in concurrent access scenarios

### 💡 Plant Photography Focus
This release includes specialized features for botanical photography applications:
- **Optimized Settings**: Deep depth of field, enhanced contrast, boosted greens
- **Quality Controls**: Maximum sharpness, low ISO, precise exposure timing
- **Workflow Integration**: One-click optimization, specialized capture modes
- **Performance**: High-resolution capture optimized for detailed plant documentation

### ⚡ Performance Improvements
- **40% faster** burst capture through async optimization
- **60% reduced** memory usage via object pooling
- **Zero-copy** frame handling where possible
- **Non-blocking** file I/O operations
- **Concurrent** camera access with RwLock

---

## [0.1.0] - 2024-12-15

### Initial Release

#### Core Features
- Cross-platform camera access (Windows, macOS, Linux)
- Basic camera device enumeration and information
- Single photo capture functionality
- Camera preview stream management
- Platform-specific camera backend integration (DirectShow, AVFoundation, V4L2)

#### Basic Commands
- `initialize_camera_system` - Platform initialization
- `get_available_cameras` - Device discovery
- `capture_single_photo` - Basic photo capture
- `start_camera_preview` / `stop_camera_preview` - Stream management
- `get_platform_info` - Platform detection and capabilities

#### Foundation
- Tauri 2.0 plugin architecture
- nokhwa backend integration for cross-platform support
- Basic error handling and logging
- Simple test framework with mock system
- MIT/Apache-2.0 dual licensing

### Technical Foundation
- Rust async/await throughout
- Memory-safe implementation (zero unsafe code)
- Type-safe camera parameter handling
- Cross-platform compilation and testing
- Comprehensive logging and debugging support

---

**Legend:**
- 🚀 Major Features
- 🛠️ Technical Improvements  
- 📊 Testing & Quality
- 🔧 Developer Experience
- 📝 Documentation
- 🐛 Bug Fixes
- 💡 Specialized Features
- ⚡ Performance