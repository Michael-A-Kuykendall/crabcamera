#Spell: WebRTCLibrarySelection
^ Intent: choose an implementation path that achieves production WebRTC with minimum risk under CrabCamera constraints.

@ LibraryDecision
  : (requirements, constraints, ecosystem_options) -> (selected_path, rationale)
  ! Guarantee: selected_path is either "webrtc-rs" (or a specific pure-Rust crate family) or "manual core protocols".
  ! Guarantee: decision includes a concrete integration boundary with src/webrtc/ modules.
  ~ Assumption: the target is WebRTC 1.0 interoperable with modern browsers.
  ~ Assumption: STUN/TURN usage is permitted as network infrastructure, not as a software dependency.
  - Exclusion: does not implement SDP/ICE/DTLS/SRTP itself.
  - Exclusion: does not add an SFU/MCU.
  > WebRTCDependencyPolicy
