#Spell: WebRTCRTPPacketizeH264
^ Intent: packetize H.264 Annex B access units into RTP payloads compliant with RFC 6184.

@ H264RTP : (annexb_access_unit) → rtp_payloads

! Guarantee: NAL unit fragmentation uses FU-A when needed.
! Guarantee: STAP-A aggregation is either supported explicitly or excluded.
! Guarantee: MTU constraints are respected.

~ Assumption: MTU is configured (default 1200 bytes payload budget).

- Exclusion: does not implement STAP-A unless explicitly required.
- Exclusion: does not accept length-prefixed AVCC input.

> WebRTCRTP