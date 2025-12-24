#Spell: WebRTCH264EncoderBridge

^ Intent: integrate openh264 output into WebRTC-ready access units with correct keyframe signaling.

@ H264
  : (raw_frame, encode_settings) -> (annexb_access_unit, is_keyframe, pts, dts?)

! Guarantee: access units contain necessary SPS/PPS before first IDR as required by negotiation strategy.
! Guarantee: keyframe detection is derived from NAL types, not heuristics.
! Guarantee: H264WebRTCEncoder produces Annex B format access units.
! Guarantee: frame_type indicates Keyframe for IDR/I frames.
! Guarantee: timestamps are preserved from input frames.

~ Assumption: encoder can be configured to emit IDR at a reasonable interval.

- Exclusion: does not implement RTCP feedback-driven keyframe requests.
- Exclusion: does not parse MP4.

> WebRTCRealEncoding
