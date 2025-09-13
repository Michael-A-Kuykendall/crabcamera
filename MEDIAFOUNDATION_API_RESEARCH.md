# MediaFoundation Camera Control APIs - Research Summary

## 🔍 **API INTERFACE MAPPING**

### **Primary Interfaces for Camera Control**

#### **1. IAMCameraControl Interface**
```rust
// Available through windows crate
use windows::Win32::Media::DirectShow::{
    IAMCameraControl,
    CameraControlProperty,
    CameraControlFlags,
};

// Control Properties Available:
pub enum CameraControlProperty {
    Pan,           // Camera pan
    Tilt,          // Camera tilt  
    Roll,          // Camera roll
    Zoom,          // Camera zoom
    Exposure,      // Exposure time
    Iris,          // Aperture/iris
    Focus,         // Focus distance
}

// Control Flags:
pub enum CameraControlFlags {
    Auto,          // Automatic control
    Manual,        // Manual control
}
```

#### **2. IAMVideoProcAmp Interface**
```rust
// Video processing amplifier controls
use windows::Win32::Media::DirectShow::{
    IAMVideoProcAmp,
    VideoProcAmpProperty,
    VideoProcAmpFlags,
};

// Video Processing Properties:
pub enum VideoProcAmpProperty {
    Brightness,           // Brightness adjustment
    Contrast,            // Contrast adjustment  
    Hue,                 // Hue adjustment
    Saturation,          // Saturation adjustment
    Sharpness,           // Sharpness adjustment
    Gamma,               // Gamma correction
    ColorEnable,         // Color enable/disable
    WhiteBalance,        // White balance temperature
    BacklightCompensation, // Backlight compensation
    Gain,                // Gain control
}
```

---

## 📐 **CONTROL MAPPING STRATEGY**

### **CrabCamera → MediaFoundation Mapping**

| CrabCamera Control | MediaFoundation API | Property | Value Range | Notes |
|-------------------|-------------------|----------|-------------|--------|
| `auto_focus: bool` | `IAMCameraControl` | `Focus` | `Auto/Manual` | Flag-based |
| `focus_distance: f32` | `IAMCameraControl` | `Focus` | Device-specific | 0.0=inf, 1.0=close |
| `auto_exposure: bool` | `IAMCameraControl` | `Exposure` | `Auto/Manual` | Flag-based |
| `exposure_time: f32` | `IAMCameraControl` | `Exposure` | Device-specific | Seconds |
| `white_balance: enum` | `IAMVideoProcAmp` | `WhiteBalance` | 2500-10000K | Kelvin temp |
| `brightness: f32` | `IAMVideoProcAmp` | `Brightness` | Device-specific | -1.0 to 1.0 |
| `contrast: f32` | `IAMVideoProcAmp` | `Contrast` | Device-specific | -1.0 to 1.0 |
| `saturation: f32` | `IAMVideoProcAmp` | `Saturation` | Device-specific | -1.0 to 1.0 |
| `sharpness: f32` | `IAMVideoProcAmp` | `Sharpness` | Device-specific | -1.0 to 1.0 |
| `zoom: f32` | `IAMCameraControl` | `Zoom` | Device-specific | Zoom factor |

### **Unsupported Controls** (for now)
- `aperture: f32` - Requires `Iris` property (limited camera support)
- `iso_sensitivity: u32` - Requires `Gain` property (complex mapping)
- `noise_reduction: bool` - No direct MediaFoundation equivalent
- `image_stabilization: bool` - Hardware-dependent, limited API support

---

## 🔧 **IMPLEMENTATION APPROACH**

### **1. Device Interface Discovery**
```rust
impl MediaFoundationControls {
    pub fn new(device_index: u32) -> Result<Self, CameraError> {
        // Step 1: Create Media Foundation device
        let media_source = Self::create_media_source(device_index)?;
        
        // Step 2: Query for control interfaces
        let camera_control: Option<IAMCameraControl> = 
            media_source.cast().ok();
        let video_proc_amp: Option<IAMVideoProcAmp> = 
            media_source.cast().ok();
            
        Ok(MediaFoundationControls {
            media_source,
            camera_control,
            video_proc_amp,
        })
    }
}
```

### **2. Control Value Mapping**
```rust
impl MediaFoundationControls {
    fn normalize_range(value: f32, min: i32, max: i32) -> i32 {
        // Convert -1.0..1.0 to device-specific range
        let range = max - min;
        let normalized = (value + 1.0) / 2.0; // 0.0..1.0
        min + (normalized * range as f32) as i32
    }
    
    fn denormalize_range(value: i32, min: i32, max: i32) -> f32 {
        // Convert device range back to -1.0..1.0
        let range = max - min;
        let normalized = (value - min) as f32 / range as f32;
        (normalized * 2.0) - 1.0
    }
}
```

### **3. White Balance Temperature Mapping**
```rust
fn white_balance_to_kelvin(wb: &WhiteBalance) -> i32 {
    match wb {
        WhiteBalance::Auto => -1,        // Use auto mode
        WhiteBalance::Incandescent => 2700,
        WhiteBalance::Fluorescent => 4200,
        WhiteBalance::Daylight => 5500,
        WhiteBalance::Flash => 5500,
        WhiteBalance::Cloudy => 6500,
        WhiteBalance::Shade => 7500,
        WhiteBalance::Custom(temp) => *temp,
    }
}
```

---

## 🧪 **CAPABILITY DETECTION STRATEGY**

### **Runtime Capability Testing**
```rust
impl MediaFoundationControls {
    pub fn get_supported_controls(&self) -> CameraCapabilities {
        let mut caps = CameraCapabilities::default();
        
        // Test camera control support
        if let Some(ref camera_ctrl) = self.camera_control {
            caps.supports_manual_focus = self.test_control_support(
                camera_ctrl, 
                CameraControlProperty::Focus
            );
            caps.supports_manual_exposure = self.test_control_support(
                camera_ctrl, 
                CameraControlProperty::Exposure
            );
            caps.supports_zoom = self.test_control_support(
                camera_ctrl, 
                CameraControlProperty::Zoom
            );
        }
        
        // Test video processing support
        if let Some(ref video_proc) = self.video_proc_amp {
            caps.supports_white_balance = self.test_video_proc_support(
                video_proc, 
                VideoProcAmpProperty::WhiteBalance
            );
            // ... test other properties
        }
        
        caps
    }
    
    fn test_control_support(&self, ctrl: &IAMCameraControl, prop: CameraControlProperty) -> bool {
        unsafe {
            let mut min = 0i32;
            let mut max = 0i32;
            let mut step = 0i32;
            let mut default = 0i32;
            let mut flags = CameraControlFlags::Auto;
            
            ctrl.GetRange(prop, &mut min, &mut max, &mut step, &mut default, &mut flags)
                .is_ok()
        }
    }
}
```

---

## ⚠️ **POTENTIAL CHALLENGES & SOLUTIONS**

### **Challenge 1: COM Interface Lifetime Management**
**Problem**: MediaFoundation uses COM interfaces that need proper cleanup
**Solution**: RAII pattern with Drop implementation
```rust
impl Drop for MediaFoundationControls {
    fn drop(&mut self) {
        // COM interfaces automatically released when dropped
        // Windows crate handles this
    }
}
```

### **Challenge 2: Device-Specific Value Ranges**
**Problem**: Each camera has different min/max ranges for controls
**Solution**: Query ranges at initialization and normalize
```rust
struct ControlRange {
    min: i32,
    max: i32,
    step: i32,
    default: i32,
}

impl MediaFoundationControls {
    fn get_control_range(&self, property: CameraControlProperty) -> Option<ControlRange> {
        // Query actual device ranges
    }
}
```

### **Challenge 3: Unsupported Controls**
**Problem**: Not all cameras support all controls
**Solution**: Graceful degradation and capability reporting
```rust
pub fn apply_controls(&mut self, controls: &CameraControls) -> Result<Vec<String>, CameraError> {
    let mut unsupported = Vec::new();
    
    if let Some(focus) = controls.focus_distance {
        match self.set_focus(focus) {
            Ok(_) => {},
            Err(_) => unsupported.push("focus_distance".to_string()),
        }
    }
    
    // Return list of unsupported controls for user feedback
    Ok(unsupported)
}
```

---

## 📋 **IMPLEMENTATION CHECKLIST**

### **Core APIs to Implement**
- [ ] **IAMCameraControl Interface** - Focus, exposure, zoom
- [ ] **IAMVideoProcAmp Interface** - Brightness, contrast, saturation, white balance
- [ ] **Control Range Query** - GetRange() for each property
- [ ] **Control Value Setting** - Set() with proper flags
- [ ] **Control Value Getting** - Get() for current values
- [ ] **Capability Detection** - Test which controls are supported

### **Value Mapping Functions**
- [ ] **Normalize Ranges** - Convert device ranges to -1.0..1.0
- [ ] **White Balance Mapping** - Enum to Kelvin temperature
- [ ] **Focus Distance Mapping** - 0.0=infinity to device units
- [ ] **Exposure Time Mapping** - Seconds to device units
- [ ] **Error Handling** - Unsupported control graceful handling

### **Integration Points**
- [ ] **Device Discovery** - Find MediaFoundation device from camera index
- [ ] **Interface Querying** - Test which interfaces are available
- [ ] **Resource Management** - Proper COM interface cleanup
- [ ] **Error Propagation** - MediaFoundation errors to CameraError
- [ ] **Thread Safety** - Ensure controls work across async boundaries

---

## 🎯 **SUCCESS CRITERIA FOR RESEARCH PHASE**

**✅ Research Complete When:**
- [x] **API Interfaces Identified** - IAMCameraControl, IAMVideoProcAmp  
- [x] **Control Mapping Planned** - Each CrabCamera control → MediaFoundation property
- [x] **Value Conversion Strategy** - Device ranges ↔ normalized ranges
- [x] **Capability Detection Method** - Runtime testing approach
- [x] **Error Handling Strategy** - Unsupported controls, COM errors
- [x] **Implementation Approach** - RAII, proper COM management

**🚀 Ready for Implementation Phase!**

---

**NEXT STEP**: Move to module structure creation - organize the Windows platform code for the new MediaFoundation integration.