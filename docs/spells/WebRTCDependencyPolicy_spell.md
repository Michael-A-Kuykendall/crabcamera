#Spell: WebRTCDependencyPolicy
^ Intent: enforce CrabCamera hard constraints for any WebRTC implementation and dependency set.

@ DependencyGate
  : (crate_manifest, license_set) -> decision
  ! Guarantee: only MIT/Apache-2.0 compatible dependencies are permitted.
  ! Guarantee: no FFmpeg/GStreamer/libav or external runtime binaries are introduced.
  ! Guarantee: the resulting system remains a single-binary Tauri plugin.
  ~ Assumption: existing C bindings (openh264, libopus_sys) remain acceptable.
  - Exclusion: does not select a specific WebRTC library.
  - Exclusion: does not implement any WebRTC protocol logic.
  > WebRTCLibrarySelection
