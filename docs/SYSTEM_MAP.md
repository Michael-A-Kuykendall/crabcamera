# 🗺️ CrabCamera Systemic Map

This document serves as the authoritative architectural map of the codebase. It details every component, data pathway, and control surface in the system. It is designed to prove that the system is a cohesive, implemented reality, not a collection of "vaporware" stubs.

## 🏗️ 1. High-Level Architecture

The system is layered to enforce separation of concerns and platform independence.

```mermaid
graph TD
    User[Frontend / User App] -->|Invokes| Commands[Tauri Commands <br/> src/commands/*]
    Commands -->|Validates via| Registry[System Registry <br/> src/registry.rs]
    Commands -->|Manages| Manager[Platform Manager <br/> src/platform/manager.rs]
    
    subgraph "Platform Abstraction Layer"
        Manager -->|Dispatches| Win[Windows Backend <br/> src/platform/windows/*]
        Manager -->|Dispatches| Mac[MacOS Backend <br/> src/platform/macos.rs]
        Manager -->|Dispatches| Linux[Linux Backend <br/> src/platform/linux.rs]
        
        Win -->|MediaFoundation| HW_Win[Windows Hardware]
        Mac -->|AVFoundation| HW_Mac[MacOS Hardware]
        Linux -->|V4L2| HW_Linux[Linux Hardware]
    end

    subgraph "Processing Pipeline"
        HW_Win & HW_Mac & HW_Linux -->|Raw Frame| Pipeline[Frame Pipeline]
        Pipeline -->|Analyze| Qual[Quality Engine <br/> src/quality/*]
        Pipeline -->|Encode| Rec[Recording Engine <br/> src/recording/*]
        Qual -->|Metrics| Commands
    end
```

---

## 🌊 2. Data Flow: The Frame Lifecycle

This diagram traces the exact path of a single video frame from hardware generation to user consumption. This is the critical "Real-time" pathway.

```mermaid
sequenceDiagram
    participant HW as Hardware Camera
    participant Drv as Platform Driver
    participant Mgr as Camera Manager
    participant Qual as Quality Validator
    participant App as Frontend App

    Note over HW, App: 1. Capture Phase
    HW->>Drv: Generate Raw Frame (YUYV/MJPEG)
    Drv->>Drv: Decode to RGB (if needed)
    Drv->>Mgr: Push CameraFrame (Arc<Mutex>)
    
    Note over Mgr, App: 2. Processing Phase
    Mgr->>Qual: Analyze(Frame)
    Qual-->>Mgr: QualityMetrics (Blur/Exposure)
    
    opt Quality Gate
        Mgr->>Mgr: Check Thresholds
    end
    
    Note over Mgr, App: 3. Consumption Phase
    Mgr->>App: Emit Event ("frame-captured", Frame + Metrics)
    Mgr-->>App: Return Result (for Command-based capture)
```

---

## 🧩 3. Component Deep Dive

### 3.1 Platform Layer (`src/platform/`)
The heart of the abstraction. Unlike previous "vaporware" iterations, this layer now contains distinct, compilation-enforced implementations for each OS.

*   **`manager.rs`**: The "Router". It holds the `CAMERA_REGISTRY` (a `HashMap` of active cameras) and handles thread-safe access to them. It enforces the "One Driver, Multiple Consumers" model using `Arc<Mutex<PlatformCamera>>`.
*   **`windows/`**:
    *   **Status**: `Implemented`.
    *   **Mechanism**: Uses custom COM object wrappers around Windows Media Foundation.
    *   **Controls**: Supports Manual Focus, Exposure, White Balance via `IAMCameraControl` and `IAMVideoProcAmp` interfaces.
*   **`macos.rs`**:
    *   **Status**: `Beta`.
    *   **Mechanism**: Uses `objc` messaging to talk directly to AVFoundation.
    *   **Controls**: Implements `AVCaptureDevice` locking and configuration for Focus/Exposure.
*   **`linux.rs`**:
    *   **Status**: `Beta`.
    *   **Mechanism**: Uses `v4l` crate to interact with Video4Linux2.
    *   **Controls**: Basic implementations for V4L2 controls.

### 3.2 Command Layer (`src/commands/`)
The public API. Every function here is exposed to the Tauri frontend.

*   **`capture.rs`**:
    *   **Real-time**: `start_camera_preview`, `stop_camera_preview`, `set_frame_callback`.
    *   **One-shot**: `capture_single_photo`, `capture_with_quality_retry`.
    *   **Lifecycle**: `release_camera` (Critical for resource cleanup).
*   **`quality.rs`**: Exposes the logic from `src/quality` to allow the frontend to ask "Is this frame blurry?" without re-implementing the math in JS.

### 3.3 Quality Engine (`src/quality/`)
A purely mathematical layer for image analysis.

*   **`blur.rs`**: Implements Laplacian Variance algorithm to detect edge sharpness.
*   **`exposure.rs`**: Implements Histogram Analysis to detect clipped highlights or crushed shadows.
*   **`smart_trigger.rs`**: A state machine that watches a stream of frames and only "triggers" a capture when quality metrics stabilize (e.g., "Wait until focus settles").

---

## 🗺️ 4. The System Registry (Source of Truth)

The file `src/registry.rs` contains the **compile-time enforced** map of every feature.

### Validated Capabilities
| ID | Feature | Status | Location | Verified? |
|----|---------|--------|----------|-----------|
| `capture.single` | Single Photo Capture | ✅ Implemented | `src/commands/capture.rs` | **YES** (Hardware Test) |
| `capture.sequence` | Burst Sequence Capture | ✅ Implemented | `src/commands/capture.rs` | **YES** (Code Review) |
| `capture.preview` | Live Preview Stream | ✅ Implemented | `src/commands/capture.rs` | **YES** (Code Review) |
| `controls.focus` | Manual Focus | 🚧 Beta | `src/platform/mod.rs` | **YES** (Windows/Mac only) |
| `controls.exposure` | Exposure Control | 🚧 Beta | `src/platform/mod.rs` | **YES** (Windows/Mac only) |
| `quality.blur` | Blur Analysis | ✅ Implemented | `src/quality/blur.rs` | **YES** (Unit Tests) |
| `quality.exposure` | Exposure Analysis | ✅ Implemented | `src/quality/exposure.rs` | **YES** (Unit Tests) |
| `platform.windows` | Windows Driver | ✅ Implemented | `src/platform/windows/` | **YES** (Hardware Test) |
| `platform.macos` | MacOS Driver | 🚧 Beta | `src/platform/macos.rs` | **PENDING** (Hardware Test) |
| `platform.linux` | Linux Driver | 🚧 Beta | `src/platform/linux.rs` | **PENDING** (Hardware Test) |

## ✅ Trust Verification
To verify this map is accurate, run the system integrity test suite:

```bash
cargo test --lib registry
```

This ensures that every entry in the `SystemRegistry` points to a real, compilable symbol in the codebase.
