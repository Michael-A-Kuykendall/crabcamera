# CrabCamera v0.4.1 Work Plan

**Created**: December 14, 2025  
**Status**: In Progress  
**Branch**: `fix/macos-compatibility-v0.4.1`

## Context

CrabCamera has 100+ stars but until today, the camera capture has never been functionally verified with real hardware. The Windows implementation was returning gray frames because:

1. `start_stream()` was a no-op (didn't call nokhwa's `open_stream()`)
2. nokhwa returns MJPEG data even when RGB is requested - we weren't decoding it
3. No warmup frames were being discarded for camera stabilization

**Fixed in commit `f70c62b`**: Camera now works on Windows with OBSBOT Tiny 4K.

---

## Phase 1: Verify All Existing Commands (PRIORITY)

Before adding features, every existing command must be tested with real hardware.

### Camera Commands to Audit

| Command | File | Status | Notes |
|---------|------|--------|-------|
| `initialize_camera_system` | `commands/init.rs` | ⬜ | |
| `get_available_cameras` | `commands/init.rs` | ⬜ | |
| `get_system_diagnostics` | `commands/init.rs` | ⬜ | NEW in v0.4.1 |
| `capture_single_photo` | `commands/capture.rs` | ⬜ | |
| `capture_photo_sequence` | `commands/capture.rs` | ⬜ | |
| `capture_with_quality_retry` | `commands/capture.rs` | ⬜ | |
| `start_camera_preview` | `commands/capture.rs` | ⬜ | |
| `stop_camera_preview` | `commands/capture.rs` | ⬜ | |
| `release_camera` | `commands/capture.rs` | ⬜ | |
| `save_frame_to_disk` | `commands/capture.rs` | ⬜ | |
| `save_frame_compressed` | `commands/capture.rs` | ⬜ | |
| `get_camera_controls` | `commands/advanced.rs` | ⬜ | |
| `set_camera_controls` | `commands/advanced.rs` | ⬜ | |
| `test_camera_capabilities` | `commands/advanced.rs` | ⬜ | |
| `capture_burst` | `commands/advanced.rs` | ⬜ | |
| `check_camera_permission` | `commands/permissions.rs` | ⬜ | |
| `request_camera_permission` | `commands/permissions.rs` | ⬜ | |

### Quality Module

| Function | File | Status | Notes |
|----------|------|--------|-------|
| `validate_frame` | `quality/validator.rs` | ⬜ | |
| `check_blur` | `quality/blur.rs` | ⬜ | |
| `check_exposure` | `quality/exposure.rs` | ⬜ | |

### Focus Stack Module

| Command | File | Status | Notes |
|---------|------|--------|-------|
| `capture_focus_stack` | `commands/focus_stack.rs` | ⬜ | Requires manual focus |
| `align_stack` | `focus_stack/align.rs` | ⬜ | |
| `merge_stack` | `focus_stack/merge.rs` | ⬜ | |

### WebRTC Module

| Command | File | Status | Notes |
|---------|------|--------|-------|
| `start_webrtc_stream` | `commands/webrtc.rs` | ⬜ | |
| `stop_webrtc_stream` | `commands/webrtc.rs` | ⬜ | |

---

## Phase 2: Cross-Platform Fixes

### macOS
- [ ] Verify MJPEG decode in `platform/macos.rs` capture
- [ ] Test with real macOS hardware (need CI or contributor)
- [ ] Verify block syntax fix compiles

### Linux  
- [ ] Verify MJPEG decode in `platform/linux.rs` capture
- [ ] Test with real Linux hardware (need CI or contributor)
- [ ] Verify V4L2 backend works

### Resolution Handling
- [ ] Investigate why 720p returns gray frames but 4K MJPEG works
- [ ] Decide: always use native resolution, or fix resolution switching
- [ ] Document supported resolutions per platform

---

## Phase 3: Release Preparation

- [ ] Update CHANGELOG.md with all verified features
- [ ] Update README.md with tested hardware list
- [ ] Create release notes
- [ ] Tag v0.4.1
- [ ] Publish to crates.io
- [ ] Close PRs #4 and #5 with attribution

---

## Test Hardware

| Device | Platform | Status |
|--------|----------|--------|
| OBSBOT Tiny 4K | Windows | ✅ Working (native 4K MJPEG) |
| OBS Virtual Camera | Windows | ⬜ Untested |
| Built-in Webcam | macOS | ⬜ Need tester |
| USB Webcam | Linux | ⬜ Need tester |

---

## Commands Reference

Test commands (run from repo root):

```bash
# Direct capture (known working)
cargo run --example direct_capture

# Full integration test
cargo run --example quick_test

# Raw nokhwa debugging  
cargo run --example raw_nokhwa_test

# Run all unit tests
cargo test
```
