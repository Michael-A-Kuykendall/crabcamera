# CrabCamera Roadmap

CrabCamera is the **first production-ready desktop camera + audio plugin** for Tauri applications.
Its mission is **invisible camera infrastructure**: drop it in, it works.

## Current Status: v0.8.5 — Honest, Audited, Production-Ready ✅

### Core Features (v0.1.0–0.5.0) — Shipped
- ✅ Cross-platform camera capture (Windows/macOS/Linux)
- ✅ Professional camera controls (focus, exposure, white balance)
- ✅ Photo capture with metadata
- ✅ Video recording with H.264 encoding
- ✅ Audio capture and synchronization
- ✅ Tauri plugin integration
- ✅ Property-based testing framework
- ✅ Published to crates.io

### Advanced Features (v0.6.0–0.8.5) — Shipped
- ✅ **Headless Operation**: Complete CLI toolkit for servers and automation
- ✅ **Production CLI**: `crabcamera-cli` for command-line camera operations
- ✅ **Session Management**: Programmatic camera/audio lifecycle control
- ✅ **Enhanced Timing**: Improved PTS clock with nanosecond precision
- ✅ **Muxide Integration**: v0.1.3 with validation and metadata support
- ✅ **Smart Trigger**: Intelligent capture automation based on quality stability
- ✅ **Invariant PPT Framework**: Runtime correctness guarantees (40+ checks)
- ✅ **Comprehensive Testing**: 85+ lib tests, property-based tests, integration tests
- ✅ **Cross-Platform Binaries**: Native headless binaries for all platforms
- ✅ **Feature Registry**: Compile-time + runtime manifest of every capability status
- ✅ **Constants Module**: Zero magic numbers — every literal is a named, documented const
- ✅ **Image Analysis CLI**: `analyze-image` command for blur/exposure scoring from files
- ✅ **Honest Error Returns**: All platform stubs return `Err` instead of silent defaults
- ✅ **Audited Codebase**: Full clippy pedantic pass — no silent panics, no fake implementations

## Next: v0.9.0 — Performance & Observability

### High Priority
- [ ] **Performance Metrics on Windows**: Implement `get_performance_metrics` for Windows via
  PDH/ETW — currently returns `UnsupportedOperation` on Windows (Linux/macOS complete)
- [ ] **`LazyLock` Migration**: Replace 5 `lazy_static!` usages with `std::sync::LazyLock`
  (stabilized in Rust 1.80, eliminates the `lazy_static` dependency)
- [ ] **`# Errors` Doc Sections**: Add rustdoc `# Errors` sections to all 108 `Result`-returning
  public functions — improves crates.io documentation completeness
- [ ] **Performance Benchmarks**: Establish Criterion baseline metrics for capture, encode, and
  focus-stack operations; catch regressions in CI

### Medium Priority
- [ ] **Multi-Camera Management**: Advanced switching, simultaneous capture, and mixing
- [ ] **Professional Audio**: Spatial audio, multi-channel support, voice activity detection
- [ ] **`From` Trait Casts**: Replace `u8 as f32` / `u8 as u32` with infallible `From::from()`
  at 14 sites (Clippy already flags these; safe mechanical change)
- [ ] **Wildcard Import Cleanup**: Replace 15 `use crate::constants::*` with explicit imports
  in modules that only use a small subset of constants

### Lower Priority
- [ ] **Live Streaming**: RTMP/RTSP output for broadcasting
- [ ] **Color Grading**: Professional color correction and LUT support
- [ ] **Timecode Synchronization**: SMPTE timecode for professional production
- [ ] **`format!` Inline Variables**: Apply `cargo clippy --fix` to modernize 223 format string
  style warnings (cosmetic, automated, zero risk)

## Future Possibilities (v1.0+)

### AI & Machine Learning
- [ ] **AI-Powered Auto-Framing**: Subject detection and automatic camera repositioning suggestions
- [ ] **Scene Recognition**: Automatic camera settings based on detected content type
- [ ] **Subject Tracking**: Follow moving subjects across frames automatically
- [ ] **Smart Cropping**: Automatic composition optimization based on rule of thirds
- [ ] **Plant/Specimen Photography Mode**: Botanical and macro-photography optimization presets

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
- **v0.8.5**: Full clippy pedantic audit — honest error returns, float epsilon comparisons,
  static-method refactors, assert! idiom, targeted cast allows with safety proofs
- **v0.8.4**: Feature Registry + Constants module; zero magic numbers across 35 source files
- **v0.8.1**: Smart Trigger + Invariant Superhighway framework (40+ runtime checks)
- **v0.6.0**: Complete headless operation with production CLI
- **Muxide Integration**: Powers MP4 recording with professional validation
- **Quality Assurance**: 85+ lib tests, property-based, and integration tests
- **Cross-Platform Success**: Unified API on Windows, macOS, Linux
- **Production Adoption**: Used in real applications with professional requirements

## Governance
- **Lead Maintainer:** Michael A. Kuykendall
- Contributions are welcome via Pull Requests
- The roadmap preserves the infrastructure-only mission
- All PRs require maintainer review and approval