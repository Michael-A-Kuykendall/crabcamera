#Spell: WebRTCRealEncoding

^ Intent: bridge CrabCamera capture pipelines into encoded frames suitable for WebRTC RTP packetization.

@ VideoPipeline
  : (raw_frames) -> (h26x_or_av1_frames, timing)
@ AudioPipeline
  : (pcm_samples) -> (opus_packets, timing)

! Guarantee: encoded outputs are directly consumable by RTP packetizers.
! Guarantee: timing is derived from capture clocks and produces monotonic PTS (and DTS when needed).
! Guarantee: H264WebRTCEncoder and OpusWebRTCEncoder provide encoded outputs.
! Guarantee: timestamps are preserved from capture sources.

~ Assumption: nokhwa produces frames at target fps.
~ Assumption: cpal produces PCM at expected sample rate.

- Exclusion: does not implement encoder internals.
- Exclusion: does not write MP4.

> WebRTCH264EncoderBridge
> WebRTCOpusEncoderBridge
