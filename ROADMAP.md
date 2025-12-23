# CrabCamera Roadmap

CrabCamera is the **first production-ready desktop camera + audio plugin** for Tauri applications.
Its mission is **invisible camera infrastructure**: drop it in, it works.

## Current Status: v0.6.0 - Headless Operation Complete ✅

### Core Features (v0.1.0-0.5.0)
- ✅ Cross-platform camera capture (Windows/macOS/Linux)
- ✅ Professional camera controls (focus, exposure, white balance)
- ✅ Photo capture with metadata
- ✅ Video recording with H.264 encoding
- ✅ Audio capture and synchronization
- ✅ Tauri plugin integration
- ✅ Property-based testing framework
- ✅ Published to crates.io

### Advanced Features (v0.6.0)
- ✅ **Headless Operation**: Complete CLI toolkit for servers and automation
- ✅ **Production CLI**: `crabcamera-cli` for command-line camera operations
- ✅ **Session Management**: Programmatic camera/audio lifecycle control
- ✅ **Enhanced Timing**: Improved PTS clock with nanosecond precision
- ✅ **Muxide Integration**: v0.1.3 with validation and metadata support
- ✅ **WebRTC Streaming**: Enhanced peer connections and data channels
- ✅ **Invariant PPT Framework**: Runtime correctness guarantees (40+ checks)
- ✅ **Comprehensive Testing**: 157+ unit tests, 95%+ coverage
- ✅ **Cross-Platform Binaries**: Native headless binaries for all platforms

## Next Goals (v0.7.0) - AI & Professional Features

### High Priority
- [ ] **AI-Powered Auto-Framing**: Subject detection and automatic camera positioning
- [ ] **Quality Enhancement**: ML-based image quality improvement
- [ ] **Performance Benchmarks**: Establish baseline metrics for camera operations

### Medium Priority
- [ ] **Multi-Camera Management**: Advanced switching and mixing capabilities
- [ ] **Professional Audio**: Spatial audio, multi-channel support, voice activity detection
- [ ] **Real-time Quality Analysis**: Live sharpness, exposure, and color assessment
- [ ] **Enhanced Documentation**: More real-world examples and integration guides

### Lower Priority
- [ ] **Live Streaming**: RTMP/RTSP output for broadcasting
- [ ] **Color Grading**: Professional color correction and LUT support
- [ ] **Timecode Synchronization**: SMPTE timecode for professional production

## Future Possibilities (v0.8.0+)

### AI & Machine Learning
- [ ] **Scene Recognition**: Automatic camera settings based on content
- [ ] **Subject Tracking**: Follow moving subjects automatically
- [ ] **Content Analysis**: Real-time analysis of captured content
- [ ] **Smart Cropping**: Automatic composition optimization

### Professional Media Production
- [ ] **Multi-Camera Switching**: Professional switching between multiple cameras
- [ ] **Virtual Sets**: Green screen and virtual background support
- [ ] **Live Mixing**: Real-time video mixing and effects
- [ ] **Broadcast Features**: Tally lights, intercom, professional workflows

### Enterprise & Compliance
- [ ] **User Management**: Multi-user camera access and permissions
- [ ] **Audit Logging**: Comprehensive logging for compliance
- [ ] **High Availability**: Failover and redundancy features
- [ ] **GDPR Compliance**: Privacy controls and data protection

### Specialized Applications
- [ ] **Medical Imaging**: DICOM support and medical imaging workflows
- [ ] **Industrial Inspection**: Measurement tools and quality control
- [ ] **Security Surveillance**: Motion detection and alert systems
- [ ] **Education Technology**: Interactive whiteboarding and recording

## ⚠️ Pro Application Boundary

### Infrastructure vs Application Line

**SAFE ZONE (Keep Free):**
- ✅ Camera/audio device APIs and controls
- ✅ Basic capture, recording, and streaming
- ✅ Cross-platform device management
- ✅ Professional camera settings (focus, exposure, etc.)
- ✅ Headless operation and CLI tools
- ✅ Basic quality analysis and metadata

**PRO APPLICATION TERRITORY (Consider Paid):**
- ❌ **Complete GUI Applications**: Custom branded interfaces
- ❌ **Vertical Solutions**: Medical imaging software, industrial inspection apps
- ❌ **Cloud Services**: Hosted recording, storage, processing platforms
- ❌ **White-label Solutions**: Rebranded software for specific industries
- ❌ **Advanced AI Features**: When they become core product differentiators

### Boundary Indicators

**Watch for these signs that we've crossed into "pro application" territory:**

1. **Complete User Experiences**: When CrabCamera includes full application workflows
2. **Industry-Specific Features**: Medical DICOM, industrial measurements, security analytics
3. **Cloud Infrastructure**: Hosted services, storage, processing
4. **Branded Experiences**: Custom UIs, workflows, branding

**Current Assessment:** CrabCamera v0.6.0 is firmly in **infrastructure territory**. The line would be crossed with features like:
- Complete medical imaging application
- Industrial inspection software with measurement tools
- Cloud-based recording service
- Branded video production suite

## Non-Goals
- **Complete Applications** - CrabCamera is infrastructure, not end-user software
- **Cloud Services** - Focus on local, self-hosted camera functionality
- **Platform Lock-in** - Maintain cross-platform compatibility
- **Feature Bloat** - Every feature must serve the core camera infrastructure mission

---

## Recent Achievements
- **v0.6.0 Release**: Complete headless operation with production CLI
- **Muxide Integration**: Powers MP4 recording with professional validation
- **Quality Assurance**: 157+ tests with runtime invariant enforcement
- **Cross-Platform Success**: Unified API working on Windows, macOS, Linux
- **Production Adoption**: Used in real applications with professional requirements

## Governance
- **Lead Maintainer:** Michael A. Kuykendall
- Contributions are welcome via Pull Requests
- The roadmap preserves the infrastructure-only mission
- All PRs require maintainer review and approval