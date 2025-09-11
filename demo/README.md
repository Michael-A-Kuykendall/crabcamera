# 🦀 CrabCamera v0.2.0 - Plant Photography Studio Demo

**Experience the power of advanced camera controls for botanical photography!**

## 🌟 Demo Features

This interactive demo showcases CrabCamera v0.2.0's professional camera capabilities through a specialized plant photography interface.

### 🎯 Key Highlights

- **📷 Professional Camera Controls**: Manual focus, exposure, white balance, and image enhancement
- **🌿 Plant Photography Optimization**: One-click botanical photography settings
- **✨ Advanced Capture Modes**: HDR burst, focus stacking, custom burst sequences
- **⚡ Performance Monitoring**: Real-time latency, FPS, and quality metrics
- **🔧 Interactive Controls**: Live adjustment of camera parameters
- **📊 Visual Feedback**: Progress tracking, performance metrics, capture gallery

## 🚀 Running the Demo

### Option 1: Direct File Access
```bash
# Open the demo directly in your browser
open demo/plant-photography-studio.html
```

### Option 2: Local Server (Recommended)
```bash
# Serve with Python (any version)
python -m http.server 8080
# or
python3 -m http.server 8080

# Then visit: http://localhost:8080/demo/plant-photography-studio.html
```

### Option 3: VS Code Live Server
1. Install "Live Server" extension in VS Code
2. Right-click `plant-photography-studio.html`
3. Select "Open with Live Server"

## 🎮 Demo Walkthrough

### 1. Initialize Camera System
- Click **"Initialize Camera System"** to start
- Watch the console log for initialization progress
- System will detect available cameras (simulated)

### 2. Select Camera
- Choose from detected cameras in the dropdown
- "Plant Photography Camera Pro" has full advanced features
- "Standard Webcam" has basic capabilities

### 3. Basic Photography
- **📸 Capture Photo**: Take single photos
- **🌱 Optimize for Plants**: One-click botanical settings
- **▶️ Start Preview**: Begin camera preview (simulated)

### 4. Advanced Modes
- **✨ HDR Burst**: 3-photo exposure bracketed sequence
- **🔍 Focus Stack**: Multi-focus depth capture
- **🔥 Custom Burst**: Configurable burst photography

### 5. Professional Controls
- Click **"Show Advanced Controls"** to reveal:
  - **Focus Control**: Auto/manual with distance slider
  - **Exposure Control**: Manual exposure time and ISO
  - **White Balance**: Multiple preset modes
  - **Image Enhancement**: Brightness, contrast, saturation, sharpness
  - **Burst Settings**: Custom count and timing

### 6. Performance Monitoring
- Real-time metrics in the left sidebar:
  - **Capture Latency**: Time from trigger to image
  - **Actual FPS**: Current frame rate
  - **Quality Score**: Image quality assessment
  - **Memory Usage**: Current memory consumption

### 7. Gallery & Analysis
- All captured photos appear in the gallery
- Click thumbnails to view capture details
- Progress bars show burst capture status

## 🌿 Plant Photography Features

### Specialized Optimizations
The demo showcases CrabCamera's botanical photography specializations:

- **Deep Depth of Field**: f/8 aperture for sharp plant details
- **Enhanced Contrast**: 30% boost for plant structure definition
- **Green Enhancement**: 40% saturation boost for vibrant foliage
- **Maximum Sharpness**: 50% increase for botanical detail capture
- **Low ISO**: ISO 100 for noise-free quality
- **Precise Exposure**: 1/60s for steady plant documentation
- **Daylight White Balance**: Optimal for outdoor botanical work

### Workflow Integration
- **One-Click Setup**: `optimize_for_plants()` applies all settings instantly
- **HDR Capture**: Perfect for challenging lighting conditions
- **Focus Stacking**: Essential for macro botanical photography
- **Performance Monitoring**: Ensure optimal capture quality

## 🔧 Technical Implementation

### Mock API
The demo uses a mock CrabCamera API that simulates:
- Camera initialization and detection
- Advanced camera controls
- Burst capture sequences
- Performance metrics
- Error handling

### Real Implementation
In actual CrabCamera integration:
```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Initialize system
await invoke('initialize_camera_system');

// Optimize for plants
await invoke('optimize_for_plants', { deviceId: '0' });

// Capture HDR sequence
const frames = await invoke('capture_hdr_sequence', { deviceId: '0' });

// Apply manual controls
await invoke('set_camera_controls', {
    deviceId: '0',
    controls: {
        auto_focus: false,
        focus_distance: 0.5,
        auto_exposure: false,
        exposure_time: 0.008, // 1/125s
        iso_sensitivity: 400
    }
});
```

## 📱 Browser Compatibility

- **Chrome/Edge**: Full compatibility
- **Firefox**: Full compatibility  
- **Safari**: Full compatibility
- **Mobile**: Responsive design works on tablets and phones

## 🎨 UI/UX Features

### Design Elements
- **Dark Theme**: Professional photography interface
- **Color Coding**: Status indicators and feature highlights
- **Progress Feedback**: Real-time operation status
- **Responsive Layout**: Works on all screen sizes

### Interactive Elements
- **Slider Controls**: Smooth parameter adjustment
- **Real-time Updates**: Live value display
- **Button States**: Clear disabled/enabled states
- **Animation Feedback**: Visual operation confirmation

### Information Display
- **Console Logging**: Detailed operation history
- **Performance Metrics**: Live system monitoring
- **Feature Highlights**: v0.2.0 new features callouts
- **Capability Detection**: Camera feature availability

## 🚀 What's Next?

This demo represents the foundation for production plant photography applications:

1. **Integration with Real Cameras**: Connect to actual hardware
2. **Advanced Image Processing**: Real-time enhancement algorithms
3. **AI-Powered Analysis**: Automated plant health assessment
4. **Cloud Integration**: Secure botanical database storage
5. **Collaboration Tools**: Multi-user botanical documentation

## 📝 Demo Statistics

- **Total Features Demonstrated**: 25+ camera operations
- **Mock Camera Types**: 2 (Professional + Standard)
- **Capture Modes**: 5 (Single, HDR, Focus Stack, Burst, Custom)
- **Control Parameters**: 12 professional settings
- **Performance Metrics**: 7 real-time measurements
- **UI Components**: 40+ interactive elements

---

**🦀 CrabCamera v0.2.0** - Bringing professional camera control to Tauri applications.

*For real implementation, see the main CrabCamera documentation and examples.*