#Spell: WebRTCRTP
^ Intent: transform encoded frames into RTP streams and transmit them over DTLS-SRTP.

@ RTPOut : (encoded_frame, timing, stream_state) → rtp_packets
@ RTPTx  : rtp_packets → network_send

! Guarantee: H.264 uses RFC 6184 packetization rules.
! Guarantee: Opus uses RFC 7587 RTP payload format.
! Guarantee: SRTP is mandatory and negotiated via DTLS.

~ Assumption: the chosen WebRTC library provides SRTP keying via DTLS handshake.

- Exclusion: does not implement custom congestion control algorithms.
- Exclusion: does not implement retransmission beyond what WebRTC stack provides.

> WebRTCRTPPacketizeH264
> WebRTCRTPPacketizeOpus
> WebRTCSRTPSession