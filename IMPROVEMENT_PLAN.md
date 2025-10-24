# CrabCamera Improvement Implementation Plan

**Status: READY TO IMPLEMENT**

## Phase 1: Quick Wins (This Week)

### Auto-Capture with Quality Retry
- [x] Add `capture_with_quality_retry` function to `src/commands/capture.rs`
- [x] Add Tauri command handler for quality retry in `src/lib.rs`
- [x] Add unit tests for retry logic
- [x] Test with real camera - verify quality threshold works
- [x] Update documentation with new capture mode

### Configuration System
- [x] Create `src/config.rs` module
- [x] Add `config.toml` support with serde
- [x] Define `CameraConfig`, `QualityConfig`, `StorageConfig` structs
- [x] Add `Cargo.toml` dependencies (config crate)
- [x] Add config loading/saving functions
- [x] Add Tauri commands for config management
- [x] Create default `config.toml` example
- [x] Add tests for config serialization
- [x] Update README with configuration docs

## Phase 2: Core Features (Next 2 Weeks)

### Error Recovery & Device Hot-Plug
- [x] Add device monitoring module `src/platform/device_monitor.rs`
- [x] Implement Windows device polling (nokhwa-based)
- [x] Implement Linux device polling (nokhwa-based)
- [x] Implement macOS device polling (nokhwa-based)
- [x] Add device reconnection logic to capture commands
- [x] Add event callbacks for device connect/disconnect
- [x] Add Tauri commands for device monitoring
- [x] Add tests for device monitor and reconnection
- [x] 58/58 tests passing

### Focus Stacking Implementation
- [x] Create `src/focus_stack/mod.rs` module
- [x] Create `src/focus_stack/capture.rs` - multi-focus capture
- [x] Create `src/focus_stack/align.rs` - image alignment
- [x] Create `src/focus_stack/merge.rs` - sharp region merging
- [x] Add edge detection for focus quality
- [x] Add pyramid blending algorithm
- [x] Add Tauri commands for focus stacking
- [x] Create focus stack configuration options
- [x] Add tests for all modules
- [x] 78/78 tests passing

### Platform-Specific Permission Handling
- [x] Fix macOS permissions in `src/permissions.rs`
- [x] Add `objc` crate dependency (already present)
- [x] Implement `AVCaptureDevice.authorizationStatus()` wrapper
- [x] Add permission request dialog for macOS
- [x] Fix Linux permissions check with group membership
- [x] Check user group membership (video/plugdev)
- [x] Add detailed PermissionInfo structure
- [x] Add 3 permission commands
- [x] 80/80 tests passing
- [ ] Test on actual macOS device (requires Mac hardware)
- [ ] Test on actual Linux device (requires Linux system)

## Phase 3: Advanced Features (Next Month)

### MediaFoundation Integration (Windows)
- [ ] Implement `find_media_source` in `src/platform/windows/controls.rs`
- [ ] Add `MFEnumDeviceSources` API wrapper
- [ ] Add `IMFActivate` interface handling
- [ ] Query `IAMCameraControl` from MediaFoundation device
- [ ] Query `IAMVideoProcAmp` from MediaFoundation device
- [ ] Implement manual focus control
- [ ] Implement manual exposure control
- [ ] Implement white balance control
- [ ] Implement pan/tilt/zoom (if supported)
- [ ] Remove `#![allow(unused_variables)]` attribute
- [ ] Add comprehensive error handling
- [ ] Test with multiple camera types
- [ ] Document advanced control usage

### CLI Tool
- [ ] Create `src/bin/crabcamera.rs` binary
- [ ] Add `clap` crate for argument parsing
- [ ] Implement `list` command - enumerate cameras
- [ ] Implement `capture` command - single capture
- [ ] Implement `preview` command - live preview display
- [ ] Implement `burst` command - rapid captures
- [ ] Implement `config` command - view/edit config
- [ ] Add progress indicators and status output
- [ ] Add man page documentation
- [ ] Test all CLI commands
- [ ] Add CLI examples to README

### Better Test Coverage
- [ ] Add platform-specific integration tests
- [ ] Add mock camera system for Windows tests
- [ ] Add mock camera system for macOS tests
- [ ] Add mock camera system for Linux tests
- [ ] Add performance benchmarks (criterion)
- [ ] Add FPS benchmark test
- [ ] Add latency benchmark test
- [ ] Add memory usage tests
- [ ] Add device disconnect tests
- [ ] Add permission denied tests
- [ ] Add long-running session tests (memory leaks)
- [ ] Set up CI to run platform-specific tests
- [ ] Document testing requirements

## Phase 4: Performance & Polish (Next Quarter)

### Performance Optimizations
- [ ] Profile frame capture pipeline
- [ ] Replace `Vec<u8>` with `Arc<[u8]>` in CameraFrame
- [ ] Update all frame consumers to use Arc
- [ ] Add zero-copy frame sharing
- [ ] Optimize quality validator algorithms
- [ ] Add SIMD for blur detection (if applicable)
- [ ] Add parallel processing for batch operations
- [ ] Benchmark before/after performance
- [ ] Document performance characteristics
- [ ] Add performance tuning guide

### Real WebRTC Streaming
- [ ] Research WebRTC libraries (webrtc-rs vs libwebrtc FFI)
- [ ] Add chosen WebRTC dependency
- [ ] Implement video encoder (H.264/VP8)
- [ ] Add hardware encoding support
- [ ] Implement RTP packet framing
- [ ] Replace mock peer connection with real implementation
- [ ] Add STUN/TURN server configuration
- [ ] Implement signaling protocol
- [ ] Add browser example client
- [ ] Test with multiple concurrent streams
- [ ] Document WebRTC setup and usage
- [ ] Add network quality monitoring

### HDR Capture Implementation
- [ ] Create `src/hdr/mod.rs` module
- [ ] Implement exposure bracketing
- [ ] Add multiple exposure capture
- [ ] Implement HDR tone mapping
- [ ] Add ghost removal algorithm
- [ ] Add HDR merge algorithms (Debevec, Reinhard)
- [ ] Add Tauri commands for HDR capture
- [ ] Create HDR configuration options
- [ ] Test with various lighting conditions
- [ ] Document HDR workflow

## Phase 5: Future Enhancements

### Advanced Quality Analysis
- [ ] Add rule-of-thirds detection
- [ ] Add subject detection/placement analysis
- [ ] Add color harmony analysis
- [ ] Add histogram equalization suggestions
- [ ] Add ML-based quality scoring (optional)
- [ ] Add composition suggestions

### Batch Processing
- [ ] Add batch capture mode
- [ ] Add batch quality analysis
- [ ] Add batch export/conversion
- [ ] Add progress tracking for batches
- [ ] Add batch configuration profiles

### Cloud Integration (Optional)
- [ ] Add cloud storage options (S3, Azure, etc.)
- [ ] Add automatic backup
- [ ] Add cloud-based processing
- [ ] Add sharing/collaboration features

## Testing Checklist (After Each Phase)

- [ ] Run `cargo clippy --lib -- -D warnings` - 0 warnings
- [ ] Run `cargo clippy --bins -- -D warnings` - 0 warnings
- [ ] Run `cargo test --lib` - all pass
- [ ] Run `cargo test --bins` - all pass
- [ ] Run `cargo build --release` - clean build
- [ ] Test on Windows with real camera
- [ ] Test on macOS with real camera (if available)
- [ ] Test on Linux with real camera (if available)
- [ ] Update documentation
- [ ] Update CHANGELOG.md
- [ ] Git commit with descriptive message

## Dependencies to Add

**Phase 1:**
- `config = "0.14"` - Configuration management
- `toml = "0.8"` - TOML parsing

**Phase 2:**
- `objc = "0.2"` (macOS) - Objective-C runtime
- `cocoa = "0.25"` (macOS) - macOS frameworks
- `libudev = "0.3"` (Linux) - Device monitoring

**Phase 3:**
- `clap = { version = "4.5", features = ["derive"] }` - CLI parsing
- `indicatif = "0.17"` - Progress bars
- `criterion = "0.5"` - Benchmarking

**Phase 4:**
- TBD based on WebRTC library choice
- `rayon = "1.8"` - Parallel processing
- `crossbeam = "0.8"` - Advanced concurrency

**Phase 5:**
- TBD based on ML library choice (optional)

## Progress Tracking

**Total Items:** ~120 tasks
**Completed:** 0
**In Progress:** 0
**Blocked:** 0

**Current Phase:** Phase 1 - Quick Wins
**Next Milestone:** Auto-capture + Config system complete

---

## Implementation Notes

- Each checkbox represents a discrete, testable unit of work
- Test after every major change, commit frequently
- Keep cargo clippy clean throughout
- Update docs as features are implemented
- Phase 1 should take 3-5 days
- Phase 2 should take 1-2 weeks
- Phase 3 should take 2-3 weeks
- Phase 4 should take 3-4 weeks
- Total estimated time: 2-3 months for full implementation

Let's start with Phase 1! 🚀
