#Spell: WebRTCOpusEncoderBridge
^ Intent: integrate libopus output into WebRTC-ready Opus packets with correct pacing.

@ Opus
  : (pcm_window, opus_settings) -> (opus_packet, pts)

! Guarantee: Opus packets are produced at stable frame durations (e.g., 20ms) unless explicitly configured.
! Guarantee: timestamps align to 48kHz timebase semantics.

~ Assumption: Opus output is raw packets (no Ogg framing).

- Exclusion: does not implement audio resampling.
- Exclusion: does not implement echo cancellation.

> WebRTCRealEncoding
