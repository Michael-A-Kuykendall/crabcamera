# 🕵️ Real Implementation Audit & Remediation Plan

**Status**: 🚧 IN PROGRESS
**Objective**: Eliminate "theater" code (stubs, hardcoded values, ignored errors) and ensure all exposed features backed by real OS/Hardware implementations.

---

## 🟢 1. Validated & Functional (Real Code)

### ✅ Core Logic
- **Frame Types**: `CameraFrame`, `CameraFormat` logic is sound.
- **Concurrency**: `RwLock` and `Mutex` usage in `src/platform/manager.rs` is correct.
- **Command Layer**: `src/commands/*.rs` correctly delegates to platform layers.

### ✅ Windows Implementation
- **Capture**: Uses `nokhwa` (MediaFoundation backend). Confirmed working.
- **Controls**: `src/platform/windows/controls.rs` was rewritten to use real `IAMCameraControl` and `IAMVideoProcAmp` COM interfaces.
- **Tests**: `tests/windows_controls_unit_test.rs` validates real hardware query.

### ✅ Quality Analysis Engine
- **Blur Detection**: `src/quality/blur.rs` implements actual Laplacian/Sobel algorithms.
- **Exposure Analysis**: `src/quality/exposure.rs` performs real histogram and luminance calculations.
- **Smart Trigger**: Uses the above real metrics.

---

## 🔴 2. Confirmed "Theater" / Stubs (Must Fix)

These modules exist but return fake data or do nothing.

### 🐧 Linux Platform (`src/platform/linux.rs`)
| Feature | Status | Reality | Fix Strategy |
|---------|--------|---------|--------------|
| `list_cameras` | ⚠️ Partial | Queries `nokhwa` but **hardcoded** supported formats (Lines 23-29). | Query V4L2 `VIDIOC_ENUM_FMT`. |
| `get_controls` | ❌ Fake | Returns `Default::default()`. | Implement V4L2 `VIDIOC_QUERYCTRL`. |
| `apply_controls`| ❌ Fake | Returns `Ok(())` (swallows input). | Implement V4L2 `VIDIOC_S_CTRL`. |
| `is_v4l2_available` | ⚠️ Brittle | Checks `/dev/video0` existence. | Check `v4l2-ctl` or generic path existence? |

### 🍎 MacOS Platform (`src/platform/macos.rs`)
| Feature | Status | Reality | Fix Strategy |
|---------|--------|---------|--------------|
| `list_cameras` | ⚠️ Partial | Queries `nokhwa` but **hardcoded** supported formats. | Query AVFoundation formats. |
| `get_controls` | ❌ Fake | Returns `Default::default()`. | Implement AVFoundation `lockForConfiguration`. |
| `apply_controls`| ❌ Fake | Returns `Ok(())` (swallows input). | Implement AVFoundation control setting. |

### 📸 Universal Stubs
| Feature | Status | Reality | Fix Strategy |
|---------|--------|---------|--------------|
| `get_performance_metrics` | ❌ Fake | Returns default metrics on all platforms. | Implement frame delta timing or drop capabilities. |
| `test_capabilities` | ❌ Fake | Returns default capabilities. | Actually range-test the controls or query flags. |

---

## 📋 Remediation Checklist

### Phase 1: Linux Reality (V4L2)
*   [ ] **Remove Hardcoded Formats**: Update `list_cameras` to use `v4l::video::capture::Parameters` or `nokhwa` query.
*   [ ] **Implement Controls**: Map `CameraControls` struct to V4L2 ioctls (Brightness, Contrast, Exposure).
*   [ ] **Validate**: Create `linux_controls_test.rs` (requires Linux env, or strict compilation check).

### Phase 2: MacOS Reality (AVFoundation)
*   [ ] **Remove Hardcoded Formats**: Query `AVCaptureDevice.formats`.
*   [ ] **Implement Controls**: Map `CameraControls` to key-value coding (KVC) properties on `AVCaptureDevice`.

### Phase 3: Cleanup
*   [ ] Search and destroy any remaining `todo!`, `unimplemented!`, or `fixed` vectors.
*   [ ] Add `#[warn(unused_variables)]` back to files to catch silences.
