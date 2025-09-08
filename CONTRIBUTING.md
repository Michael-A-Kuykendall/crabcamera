# Contributing to CrabCamera

Thanks for your interest in contributing to the desktop camera infrastructure!

## How to Contribute
1. Fork the repo and create a branch (`git checkout -b feature/camera-enhancement`).
2. Make your changes with clear commits and platform-specific tests.
3. Test across all supported platforms (Windows/macOS/Linux) when possible.
4. Run the full test suite to ensure nothing breaks (`cargo test --all-features`).
5. Open a Pull Request against `main` with detailed description.

## Code Style
- Rust 2021 edition with `cargo fmt` and `cargo clippy` before submitting.
- Comprehensive error handling using `Result<T, CrabCameraError>`.
- All public APIs must have documentation with examples.
- Platform-specific code should be properly abstracted in `/src/platform/`.

## Contribution Scope
Features should align with the **CrabCamera philosophy**:  
- **Cross-platform compatibility**: Must work on Windows, macOS, and Linux
- **Production ready**: Memory-safe, well-tested, comprehensive error handling  
- **Invisible infrastructure**: Simple APIs that camera applications just work with
- **Native performance**: Direct hardware access without web API limitations
- **Free forever**: No features that could lead to paid tiers or restrictions

## Platform Requirements
- **Windows**: Test with DirectShow and MediaFoundation backends.
- **macOS**: Ensure AVFoundation integration works properly.
- **Linux**: Verify V4L2 compatibility across distributions.
- **Hardware**: Test with multiple camera types (USB, built-in, professional).

## Review Process
- All PRs require review and approval from the lead maintainer.
- Cross-platform compatibility will be verified before merge.
- Performance impact will be measured for camera operations.
- Hardware compatibility will be tested with multiple devices.
- Merge authority is reserved to maintain project direction and quality.

## Testing Requirements
- Unit tests for all new functions and camera operations.
- Integration tests for Tauri plugin functionality.
- Platform-specific tests for each supported operating system.
- Hardware tests with different camera models when possible.
- Performance benchmarks for capture and streaming operations.

## Camera Standards
- Follow platform-specific best practices (DirectShow, AVFoundation, V4L2).
- Implement proper resource cleanup and device release.
- Handle camera permissions appropriately for each platform.
- Support industry-standard formats and resolutions.
- Maintain backwards compatibility with existing camera hardware.

## Recognition
Contributors are acknowledged in `AUTHORS.md` after a merged PR.
Significant contributors may be invited to co-maintain platform-specific modules.