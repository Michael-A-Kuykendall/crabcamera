#Spell: WebRTCRTPPacketizeOpus
^ Intent: packetize Opus packets into RTP payloads compliant with RFC 7587.

@ OpusRTP : (opus_packet) → rtp_payload

! Guarantee: each Opus packet maps to a single RTP packet unless library dictates otherwise.
! Guarantee: RTP timestamp increments follow 48kHz clock.

~ Assumption: Opus payload sizes remain below MTU under configured bitrate.

- Exclusion: does not bundle multiple Opus frames per RTP packet unless configured.

> WebRTCRTP