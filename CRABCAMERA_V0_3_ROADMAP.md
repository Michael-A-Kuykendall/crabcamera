# CrabCamera v0.3.0 Roadmap - Focused Improvements

## 🎯 Mission: Complete the Desktop Camera Experience
**Goal**: Fix critical gaps and deliver professional desktop camera functionality without scope creep.

---

## 📋 **Task Breakdown - 3 High-Impact Features**

### **🏆 PRIORITY 1: Windows MediaFoundation Camera Controls**
**Problem**: Windows users get basic functionality while macOS/Linux get advanced controls  
**Impact**: Makes Windows a first-class citizen (majority of desktop users)

#### Tasks:
- [ ] **Research MediaFoundation Control APIs** - Map available camera control interfaces
- [ ] **Implement Windows Control Wrapper** - Create MediaFoundation control interface in `src/platform/windows.rs`
- [ ] **Add Control Parameter Mapping** - Map generic controls to MediaFoundation specifics
- [ ] **Test with Multiple Camera Types** - Verify with integrated, USB, and virtual cameras
- [ ] **Update Examples** - Ensure camera_preview.rs works with Windows controls

**Files to Modify:**
- `src/platform/windows.rs` (line 301 TODO)
- `src/platform/mod.rs` (remove Windows limitations)
- `examples/camera_preview.rs` (Windows testing)

---

### **🏆 PRIORITY 2: WebRTC Live Preview Streaming**  
**Problem**: `start_camera_preview()` exists but no actual stream display  
**Impact**: Essential for photo composition, every camera app needs this

#### Tasks:
- [ ] **Design WebRTC Architecture** - Local streaming endpoint design
- [ ] **Implement Rust WebRTC Stream** - Camera frames to WebRTC endpoint
- [ ] **Create Tauri Frontend Component** - HTML5 video element with WebRTC
- [ ] **Add Stream State Management** - Start/stop/reconnect logic
- [ ] **Cross-Platform Testing** - Verify on Windows/macOS/Linux

**Files to Create/Modify:**
- `src/streaming/mod.rs` (new module)
- `src/streaming/webrtc.rs` (WebRTC implementation)
- `examples/live_preview_demo/` (new demo)

---

### **🏆 PRIORITY 3: Auto-Capture Quality Validation**
**Problem**: Captures whatever camera gives, no quality assurance  
**Impact**: Professional results without user expertise

#### Tasks:
- [ ] **Implement Blur Detection** - Laplacian variance for sharpness detection
- [ ] **Add Exposure Validation** - Histogram analysis for over/under exposure
- [ ] **Create Auto-Retry Logic** - Automatic recapture for poor quality shots
- [ ] **Build Quality Scoring** - Combine metrics into overall quality score
- [ ] **Add Quality Configuration** - User-configurable quality thresholds

**Files to Create/Modify:**
- `src/quality/mod.rs` (new module)
- `src/quality/blur_detection.rs` (sharpness analysis)
- `src/quality/exposure_analysis.rs` (lighting validation)
- `src/commands/capture.rs` (integrate quality checks)

---

## 🚫 **Explicitly OUT OF SCOPE** (Avoid Feature Creep)
- ❌ **Video Recording** - Different market, major complexity
- ❌ **Image Editing** - Not our job, use external libraries  
- ❌ **Cloud Storage** - Infrastructure nightmare, let users handle it
- ❌ **Social Sharing** - Scope creep, not core functionality
- ❌ **Format Conversion** - Basic formats only, avoid overengineering

---

## 📅 **Development Timeline**

### **Week 1: Windows MediaFoundation Controls**
- Research and implement MediaFoundation control wrapper
- Test with multiple camera types and configurations
- Update documentation and examples

### **Week 2: WebRTC Live Preview** 
- Implement WebRTC streaming from Rust
- Create Tauri frontend preview component
- Cross-platform testing and optimization

### **Week 3: Quality Validation System**
- Build blur detection and exposure analysis
- Implement auto-retry and quality scoring
- Integration testing and user validation

### **Week 4: Polish & Release**
- Comprehensive testing across platforms
- Documentation updates and examples
- v0.3.0 release preparation

---

## 🎯 **Success Metrics**

### **Technical Goals**
- [ ] Windows camera controls match macOS/Linux functionality
- [ ] Live preview works smoothly on all platforms with <100ms latency
- [ ] Quality validation improves photo success rate by >80%
- [ ] All existing functionality remains stable

### **User Experience Goals**  
- [ ] Windows users get full professional controls
- [ ] Live preview enables proper photo composition
- [ ] Auto-quality reduces bad shots and user frustration
- [ ] Zero breaking changes for existing integrations

### **Community Goals**
- [ ] Address top user complaints from GitHub issues
- [ ] Maintain "easy to use" reputation while adding power features
- [ ] Prepare foundation for future advanced features

---

## 🚀 **Post-v0.3.0 Future Considerations**
*(Not for this release, but architectural prep)*
- Hardware-specific optimizations (Canon/Nikon DSLR support)
- Advanced HDR and focus stacking improvements  
- Plugin architecture for custom image processing
- Mobile platform exploration (React Native bindings)

---

**🎉 GOAL: v0.3.0 delivers complete desktop camera experience - professional controls on all platforms, live preview for composition, and intelligent quality assurance.**