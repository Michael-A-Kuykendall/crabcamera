# Windows MediaFoundation Camera Controls - Architecture & Implementation Plan

## 🏗️ **CURRENT STATE ANALYSIS**

### **✅ What We Have**
- **Basic Windows Support**: Camera detection and frame capture via nokhwa
- **MediaFoundation Backend**: Already using `nokhwa::utils::ApiBackend::MediaFoundation`
- **Windows Crate**: `windows = "0.58"` with `Win32_Media_MediaFoundation` features
- **Control Structure**: `CameraControls` struct with 13 professional controls
- **TODO Target**: Line 301 in `src/platform/mod.rs` - `apply_controls()` for Windows

### **❌ What's Missing**
- **Control Implementation**: Windows `apply_controls()` just logs warning and returns OK
- **MediaFoundation Interface**: No direct MF control access
- **Control Mapping**: No translation between generic controls and MF-specific APIs
- **Capability Detection**: No way to know which controls are supported

---

## 🎯 **ARCHITECTURE DESIGN**

### **Strategy: Hybrid Approach**
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   nokhwa        │    │  MediaFoundation │    │  CrabCamera     │
│   (Capture)     │    │  (Controls)      │    │  (Unified API)  │
├─────────────────┤    ├──────────────────┤    ├─────────────────┤
│ • Frame capture │    │ • Focus control  │    │ • Generic types │
│ • Resolution    │    │ • Exposure       │    │ • Error handling│
│ • Format        │    │ • White balance  │    │ • Cross-platform│
│ • Start/Stop    │    │ • Brightness     │    │ • Validation    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

**Why Hybrid?**
- **nokhwa**: Excellent for cross-platform capture, less control support
- **MediaFoundation**: Native Windows controls, complex COM interfaces  
- **Separation**: Keep capture stable while adding advanced controls

---

## 📐 **TECHNICAL ARCHITECTURE**

### **1. Control Mapping Layer**
```rust
// New file: src/platform/windows/controls.rs
pub struct MediaFoundationControls {
    device: ComPtr<IMFMediaSource>,
    camera_control: Option<ComPtr<IAMCameraControl>>,
    video_proc_amp: Option<ComPtr<IAMVideoProcAmp>>,
}

impl MediaFoundationControls {
    pub fn new(device_index: u32) -> Result<Self, CameraError> {
        // Initialize COM interfaces for camera control
    }
    
    pub fn apply_controls(&mut self, controls: &CameraControls) -> Result<(), CameraError> {
        // Map generic controls to MediaFoundation calls
    }
    
    pub fn get_controls(&self) -> Result<CameraControls, CameraError> {
        // Read current control values from MediaFoundation
    }
}
```

### **2. Control Mapping Strategy**
| Generic Control | MediaFoundation Interface | Method |
|----------------|---------------------------|--------|
| `focus_distance` | `IAMCameraControl` | `Set(CameraControl_Focus, value, Flags_Manual)` |
| `exposure_time` | `IAMCameraControl` | `Set(CameraControl_Exposure, value, Flags_Manual)` |
| `white_balance` | `IAMVideoProcAmp` | `Set(VideoProcAmp_WhiteBalance, kelvin, Flags_Manual)` |
| `brightness` | `IAMVideoProcAmp` | `Set(VideoProcAmp_Brightness, value, Flags_Manual)` |
| `contrast` | `IAMVideoProcAmp` | `Set(VideoProcAmp_Contrast, value, Flags_Manual)` |
| `saturation` | `IAMVideoProcAmp` | `Set(VideoProcAmp_Saturation, value, Flags_Manual)` |

### **3. Enhanced Windows Platform Structure**
```rust
// Modify src/platform/windows.rs
pub struct WindowsCamera {
    pub nokhwa_camera: Camera,              // For capture
    pub mf_controls: MediaFoundationControls, // For controls
    pub device_id: String,
}

impl WindowsCamera {
    pub fn new(device_id: String, format: CameraFormat) -> Result<Self, CameraError> {
        let nokhwa_camera = initialize_camera(&device_id, format)?;
        let mf_controls = MediaFoundationControls::new(device_id.parse()?)?;
        
        Ok(WindowsCamera {
            nokhwa_camera,
            mf_controls,
            device_id,
        })
    }
}
```

### **4. Platform Integration**
```rust
// Modify src/platform/mod.rs PlatformCamera enum
pub enum PlatformCamera {
    #[cfg(target_os = "windows")]
    Windows(WindowsCamera),  // Changed from nokhwa::Camera to WindowsCamera
    
    // ... other platforms unchanged
}
```

---

## 🔧 **IMPLEMENTATION TASKS**

### **Phase 1: Foundation (Week 1, Days 1-2)**

#### **Task 1.1: Research MediaFoundation APIs**
- [ ] **Study IAMCameraControl Interface** - Focus, exposure, zoom controls
- [ ] **Study IAMVideoProcAmp Interface** - Brightness, contrast, saturation, white balance
- [ ] **Research COM Interface Patterns** - Proper initialization and cleanup
- [ ] **Document Control Ranges** - Min/max values for each control type
- [ ] **Test Camera Compatibility** - Which cameras support which controls

#### **Task 1.2: Create Windows Controls Module**
- [ ] **Create `src/platform/windows/mod.rs`** - Module reorganization
- [ ] **Create `src/platform/windows/capture.rs`** - Move existing capture code
- [ ] **Create `src/platform/windows/controls.rs`** - New MediaFoundation wrapper
- [ ] **Update `src/platform/windows.rs`** - Re-export and integration
- [ ] **Add Windows Features** - Extend Cargo.toml if needed

### **Phase 2: Core Implementation (Week 1, Days 3-5)**

#### **Task 2.1: MediaFoundation Control Interface**
- [ ] **Implement MediaFoundationControls Struct** - COM interface management
- [ ] **Add Device Enumeration** - Find control interfaces for camera
- [ ] **Implement Control Mapping** - Generic CameraControls → MF APIs
- [ ] **Add Error Handling** - COM errors, unsupported controls
- [ ] **Add Resource Cleanup** - Proper COM interface release

#### **Task 2.2: WindowsCamera Integration**
- [ ] **Create WindowsCamera Struct** - Combine nokhwa + MF controls
- [ ] **Implement apply_controls()** - Route to MediaFoundation
- [ ] **Implement get_controls()** - Read current values from MF
- [ ] **Add Capability Detection** - Test which controls are supported
- [ ] **Maintain Capture Compatibility** - Ensure nokhwa still works

### **Phase 3: Platform Integration (Week 1, Days 6-7)**

#### **Task 3.1: Update Platform Layer**
- [ ] **Modify PlatformCamera Enum** - Windows(WindowsCamera) instead of Camera
- [ ] **Update apply_controls()** - Remove TODO, call WindowsCamera
- [ ] **Update get_controls()** - Return actual Windows control values
- [ ] **Update test_capabilities()** - Report real Windows capabilities
- [ ] **Add Performance Metrics** - Control latency measurement

#### **Task 3.2: Testing & Validation**
- [ ] **Create Windows Control Tests** - Unit tests for MF wrapper
- [ ] **Test Multiple Camera Types** - Integrated, USB, virtual cameras
- [ ] **Test Control Ranges** - Min/max value validation
- [ ] **Cross-Platform Testing** - Ensure macOS/Linux still work
- [ ] **Error Condition Testing** - Unsupported controls, disconnected cameras

---

## 🧪 **TESTING STRATEGY**

### **1. Unit Testing**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_windows_focus_control() {
        let mut camera = WindowsCamera::new("0".to_string(), CameraFormat::standard()).unwrap();
        
        let controls = CameraControls {
            auto_focus: Some(false),
            focus_distance: Some(0.5),
            ..Default::default()
        };
        
        assert!(camera.apply_controls(&controls).is_ok());
        
        let current = camera.get_controls().unwrap();
        assert_eq!(current.auto_focus, Some(false));
        assert!((current.focus_distance.unwrap() - 0.5).abs() < 0.1);
    }
}
```

### **2. Integration Testing**
- **Real Hardware Testing**: Multiple camera brands and models
- **Virtual Camera Testing**: OBS, ManyCam, etc.  
- **Control Compatibility**: Which cameras support which controls
- **Performance Testing**: Control application latency
- **Error Recovery**: Camera disconnect/reconnect scenarios

### **3. Mock Testing**
```rust
// For CI/CD environments without cameras
#[cfg(feature = "mock-windows")]
impl MediaFoundationControls {
    pub fn new_mock() -> Self {
        // Return mock implementation for testing
    }
}
```

---

## 🎯 **SUCCESS CRITERIA**

### **Technical Goals**
- [ ] **Windows controls match macOS/Linux functionality** - Same CameraControls interface
- [ ] **Support 8+ control types** - Focus, exposure, white balance, brightness, contrast, etc.
- [ ] **<50ms control latency** - Professional responsiveness  
- [ ] **95%+ camera compatibility** - Works with most Windows cameras
- [ ] **Zero regression** - Existing capture functionality unchanged

### **Code Quality Goals**
- [ ] **Proper COM management** - No memory leaks or interface issues
- [ ] **Comprehensive error handling** - Graceful unsupported control handling
- [ ] **Full test coverage** - Unit + integration tests for all controls
- [ ] **Documentation** - Clear examples and API docs
- [ ] **Cross-platform stability** - Other platforms unaffected

### **User Experience Goals**
- [ ] **Seamless operation** - Controls work like they do on macOS/Linux
- [ ] **Clear error messages** - When controls aren't supported
- [ ] **Capability reporting** - Users know what their camera can do
- [ ] **Professional quality** - Suitable for serious photography apps

---

## 🚀 **FUTURE EXTENSIONS** (Post-v0.3.0)

### **Advanced Features**
- **Multi-camera controls** - Simultaneous control of multiple Windows cameras
- **Hardware-specific optimizations** - Canon/Nikon DSLR integration
- **Advanced MediaFoundation** - DirectShow filters, custom processing
- **Performance profiling** - Control latency optimization

### **Professional Features**  
- **Camera profiles** - Save/restore control configurations
- **Automation** - Script-driven control sequences
- **Real-time feedback** - Live control value monitoring
- **Integration APIs** - External hardware integration

---

**🎉 OUTCOME: Windows users get the same professional camera control experience as macOS and Linux users, making CrabCamera truly cross-platform complete.**