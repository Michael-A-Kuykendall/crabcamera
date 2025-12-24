#Spell: WebRTCDependencyPolicy

^ Intent: enforce CrabCamera hard constraints for WebRTC dependencies by adding compliant crates to Cargo.toml.

@ DependencyGate
  : (crate_manifest, license_set) -> decision
  ! Guarantee: only MIT/Apache-2.0 compatible dependencies are permitted.
  ! Guarantee: no FFmpeg/GStreamer/libav or external runtime binaries are introduced.
  ! Guarantee: the resulting system remains a single-binary Tauri plugin.
  ! Guarantee: webrtc crate (Apache-2.0) added as optional dependency.
  ! Guarantee: rcgen crate (MIT) added as optional dependency for DTLS certificates.
  ! Guarantee: bytes crate (MIT) added as optional dependency for buffer handling.
  ! Guarantee: rustls crate (Apache-2.0/MIT) added as optional dependency for TLS.
  ! Guarantee: all dependencies gated behind 'webrtc' feature flag.
  ! Guarantee: no FFmpeg or GPL dependencies introduced.
  ~ Assumption: existing C bindings (openh264, libopus_sys) remain acceptable.
  ~ Assumption: webrtc-rs is the selected library (from next step).
  - Exclusion: does not select a specific WebRTC library.
  - Exclusion: does not implement any WebRTC protocol logic.
  - Exclusion: does not implement any WebRTC logic.
  - Exclusion: does not verify transitive dependencies beyond direct adds.
  > WebRTCLibrarySelection
