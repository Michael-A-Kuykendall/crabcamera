#Spell: WebRTCLibrarySelection

^ Intent: choose webrtc-rs as the implementation path for production WebRTC under CrabCamera constraints.

@ LibraryDecision
  ! Guarantee: selected_path is either "webrtc-rs" (or a specific pure-Rust crate family) or "manual core protocols".
  : (requirements, constraints, ecosystem_options) -> (selected_path, rationale)
  ! Guarantee: selected_path is "webrtc-rs".
  ! Guarantee: decision includes a concrete integration boundary with src/webrtc/ modules.
  ! Guarantee: src/webrtc/mod.rs documents the library choice and integration points.
  ! Guarantee: peer connections use webrtc::peer_connection::RTCPeerConnection.
  ! Guarantee: SDP handling via webrtc::sdp.
  ! Guarantee: ICE via webrtc::ice_transport.
  ! Guarantee: RTP/SRTP via webrtc::rtp_transceiver.
  ! Guarantee: data channels via webrtc::data_channel.
  ~ Assumption: the target is WebRTC 1.0 interoperable with modern browsers.
  ~ Assumption: STUN/TURN usage is permitted as network infrastructure, not as a software dependency.
  - Exclusion: does not implement SDP/ICE/DTLS/SRTP itself.
  - Exclusion: does not add an SFU/MCU.
  > WebRTCDependencyPolicy
