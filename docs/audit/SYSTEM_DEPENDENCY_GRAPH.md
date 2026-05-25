# System Dependency Graph & Audit Status

This document tracks the systematic audit of the `crabcamera` codebase to eliminate "vapor" (unfinished code, magic values, bad practices) and ensure structural integrity.

## Audit Methodology: Radiating Pattern
1.  **Core Definitions**: `types.rs`, `errors.rs`, `constants.rs` (Foundation)
2.  **Core Logic**: `registry.rs`, `config.rs`, `invariant_ppt.rs`
3.  **Modules**: `audio/`, `focus_stack/`, `quality/`, `recording/`, `headless/`
4.  **Platform Abstraction**: `platform/` (Hardware Interface)
5.  **Application Layer**: `commands/` (Tauri Bridge), `permissions.rs`
6.  **Entry Points**: `lib.rs`, `bin/*.rs`

## 1. Core Definitions (Foundation)
- [x] `src/types.rs`
- [x] `src/errors.rs`
- [x] `src/constants.rs`

## 2. Core Logic
- [ ] `src/registry.rs`
- [ ] `src/config.rs`
- [ ] `src/invariant_ppt.rs`
- [ ] `src/timing/mod.rs` (if exists)

## 3. Modules
### Audio
- [ ] `src/audio/mod.rs`
- [x] `src/audio/capture.rs`
- [ ] `src/audio/device.rs`
- [ ] `src/audio/encoder.rs`

### Focus Stack
- [ ] `src/focus_stack/mod.rs`
- [ ] `src/focus_stack/capture.rs`
- [ ] `src/focus_stack/align.rs`
- [ ] `src/focus_stack/merge.rs`

### Quality Analysis
- [ ] `src/quality/mod.rs`
- [ ] `src/quality/blur.rs`
- [ ] `src/quality/exposure.rs`
- [ ] `src/quality/smart_trigger.rs`
- [ ] `src/quality/validator.rs`

### Recording
- [ ] `src/recording/mod.rs`
- [ ] `src/recording/recorder.rs`
- [ ] `src/recording/encoder.rs`
- [ ] `src/recording/config.rs`

### Headless
- [ ] `src/headless/mod.rs`
- [ ] `src/headless/session.rs`
- [ ] `src/headless/controls.rs`
- [ ] `src/headless/errors.rs`
- [ ] `src/headless/types.rs`

## 4. Platform Abstraction
- [ ] `src/platform/mod.rs`
- [ ] `src/platform/manager.rs`
- [ ] `src/platform/device_monitor.rs`
- [ ] `src/platform/windows/mod.rs`
- [ ] `src/platform/windows/capture.rs`
- [ ] `src/platform/windows/controls.rs`
- [ ] `src/platform/macos.rs`
- [ ] `src/platform/linux.rs`

## 5. Application Layer (Commands)
- [ ] `src/commands/mod.rs`
- [ ] `src/commands/init.rs`
- [ ] `src/commands/capture.rs`
- [ ] `src/commands/advanced.rs`
- [ ] `src/commands/focus_stack.rs`
- [ ] `src/commands/recording.rs`
- [ ] `src/commands/quality.rs`
- [ ] `src/commands/audio.rs`
- [ ] `src/commands/config.rs`
- [ ] `src/commands/device_monitor.rs`
- [ ] `src/commands/permissions.rs`
- [ ] `src/permissions.rs`

## 6. Entry Points
- [ ] `src/lib.rs`
- [ ] `src/bin/cli.rs`
- [ ] `src/bin/headless_capture.rs`
