#Spell: WebRTCRealSDP
^ Intent: implement real SDP generation and parsing instead of mock templates.

@ RealSDP
  : sdp_string -> parsed_sdp
  ! Guarantee: SDP strings are correctly parsed into structured objects.
  ! Guarantee: structured SDP objects are correctly serialized to strings.
  ! Guarantee: all WebRTC-required SDP attributes are present and valid.
  ~ Assumption: SDP parsing is handled by webrtc library.
  - Exclusion: does not implement SDP RFC parsing itself.
  - Exclusion: does not validate SDP semantics beyond library capabilities.
  > WebRTCSignalingContract
