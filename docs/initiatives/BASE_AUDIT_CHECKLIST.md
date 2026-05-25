# Codebase Audit Checklist

This audit tracks the systematic verification of every file in `crabcamera`.  
Completing this audit guarantees consistency, stability, and idiomatic correctness.

## Audit Legend
- **[ ]**: Pending
- **[x]**: Verified & Fixed
- **[-]**: Deprecated / To Delete
- **[!]**: Contains Known Issues (Requires Refactor)

## Audit Criteria (The "Definitions of Done")
Every file must pass these checks:
1.  **No Magic Values**: All raw numbers/strings replaced with constants or config.
2.  **No Shortcuts**: No `todo!`, `unimplemented!`, or `unwrap()` (in production code).
3.  **Dependencies**: No circular imports; correct visibility (`pub`, `pub(crate)`).
4.  **Error Handling**: Returns `Result<T, CameraError>`, no panics.
5.  **Documentation**: Public items have doc comments; complex logic explained.
6.  **Tests**: Unit tests exist or are covered by integration tests.

---

## Phase 1: Core Definitions (Leaf Nodes)
These files have few/no internal dependencies. They define the vocabulary of the system.

- [x] `src/types.rs` - Core data structures (CameraFormat, CameraFrame).
- [x] `src/constants.rs` - The single source of truth for magic values.
- [x] `src/errors.rs` - The canonical error type `CameraError`.
- [x] `src/config.rs` - Configuration structs and serialization.
- [x] `src/registry.rs` - Feature flag verification and system manifest.
- [x] `src/permissions.rs` - Permission checking logic.

## Phase 2: Core Logic (Platform-Agnostic)
Pure Rust logic that implements business rules (quality, processing).

- [x] `src/quality/mod.rs` - Module definition.
- [x] `src/quality/blur.rs` - Blur detection algorithms.
- [x] `src/quality/exposure.rs` - Exposure analysis.
- [x] `src/quality/smart_trigger.rs` - Logic for auto-capture.
- [x] `src/quality/validator.rs` - Frame validation logic.
- [x] `src/recording/config.rs` - Recording-specific config.
- [x] `src/recording/encoder.rs` - Video encoding abstraction.
- [x] `src/recording/recorder.rs` - Main recording loop.
- [x] `src/recording/mod.rs` - Module definition.
- [x] `src/focus_stack/align.rs` - Image alignment.
- [x] `src/focus_stack/capture.rs` - Focus bracket capture logic.
- [x] `src/focus_stack/merge.rs` - Image merging.
- [x] `src/focus_stack/mod.rs` - Module definition.
- [x] `src/audio/capture.rs` - Audio input handling.
- [x] `src/audio/encoder.rs` - Audio encoding.
- [x] `src/audio/device.rs` - Audio device discovery.
- [x] `src/audio/mod.rs` - Module definition.

## Phase 3: Platform Integration (The "Dirty" Layer)
Code that touches the OS or hardware. Requires rigorous error handling.

- [x] `src/platform/mod.rs` - The `PlatformCamera` enum and abstraction trait.
- [x] `src/platform/manager.rs` - Camera lifecycle (connect/disconnect/reconnect).
- [x] `src/platform/device_monitor.rs` - Hot-plug detection.
- [x] `src/platform/linux.rs` - V4L2 implementation.
- [x] `src/platform/macos.rs` - AVFoundation implementation.
- [x] `src/platform/windows/mod.rs` - Module definition.
- [x] `src/platform/windows/capture.rs` - MediaFoundation implementation.
- [x] `src/platform/windows/controls.rs` - Camera property controls.

## Phase 4: Application Layer (Tauri Commands)
The bridge between the frontend and the Rust backend.
Must only coordinate Phase 2 & 3 components; no business logic allowed here.

- [ ] `src/commands/mod.rs`
- [ ] `src/commands/scan.rs` (Note: Need to verify if this exists or if `init` covers it)
- [ ] `src/commands/init.rs` - Startup and teardown.
- [ ] `src/commands/capture.rs` - Taking photos/streams.
- [ ] `src/commands/advanced.rs` - Manual controls.
- [ ] `src/commands/quality.rs` - Quality checks.
- [ ] `src/commands/recording.rs` - Video recording control.
- [ ] `src/commands/focus_stack.rs` - Focus stacking control.
- [ ] `src/commands/config.rs` - Config CRUD.
- [ ] `src/commands/device_monitor.rs` - Event streams.
- [ ] `src/commands/permissions.rs` - Permission requests.
- [ ] `src/commands/audio.rs` - Audio control.

## Phase 5: Entry Points & Testing
- [ ] `src/lib.rs` - Crate root.
- [ ] `src/bin/cli.rs` - CLI entry point.
- [ ] `src/bin/headless_capture.rs` - Headless runner.
- [ ] `src/headless/mod.rs`
- [ ] `src/headless/session.rs`
- [ ] `src/headless/controls.rs`
- [ ] `src/headless/errors.rs`
- [ ] `src/headless/types.rs`
- [ ] `src/tests/mod.rs`
- [ ] `src/testing/mod.rs`
- [ ] `src/testing/synthetic_data.rs`

---

## Audit Log
| Date | File | Auditor | Status | Notes |
|------|------|---------|--------|-------|
| 2026-03-01 |  |  |  |  |
