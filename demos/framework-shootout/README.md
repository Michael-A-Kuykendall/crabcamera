# 🎯 CrabCamera GUI Framework Shootout

## Mission: Build Identical Camera Demo in 4 Rust GUI Frameworks

This directory contains identical camera control demos built in:

1. **Iced** - Pure Rust, reactive architecture  
2. **egui** - Immediate mode GUI
3. **Slint** - Declarative markup approach
4. **Tauri** - Web frontend approach

## Demo Features (Identical Across All Frameworks)

- **Live Camera Preview** - Real webcam feed
- **Focus Control** - Slider (0-100%)
- **Exposure Controls** - ISO and shutter speed
- **White Balance** - Dropdown selection
- **Capture Button** - Save photo
- **Performance Metrics** - FPS, latency, memory

## Comparison Metrics

### Development Experience
- **Setup Time** - From zero to working demo
- **Code Complexity** - Lines of code, readability
- **Learning Curve** - Documentation quality, examples

### Runtime Performance  
- **Startup Time** - App launch speed
- **Memory Usage** - RAM consumption
- **CPU Usage** - Processing overhead
- **FPS** - UI responsiveness

### UI Quality
- **Visual Polish** - Professional appearance
- **UX Flow** - Intuitive controls
- **Platform Integration** - Native look/feel

### Camera Integration
- **CrabCamera Binding** - Ease of integration
- **Real-time Performance** - Preview latency
- **Control Responsiveness** - Slider/button feedback

## Directory Structure

```
framework-shootout/
├── iced-demo/          # Iced implementation
├── egui-demo/          # egui implementation  
├── slint-demo/         # Slint implementation
├── tauri-demo/         # Tauri implementation
├── shared/             # Shared CrabCamera integration
└── results/            # Performance comparison data
```

## Outcome

This shootout will produce a definitive white paper on Rust GUI framework selection for camera applications, informing future CrabCamera demo development.