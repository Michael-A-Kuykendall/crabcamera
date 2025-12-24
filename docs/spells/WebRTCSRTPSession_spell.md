#Spell: WebRTCSRTPSession
^ Intent: establish SRTP encryption keys from DTLS handshake and apply SRTP to RTP media.

@ SRTP : (dtls_state) → (srtp_keys, srtp_context)

! Guarantee: media is encrypted with SRTP for all peers.
! Guarantee: failure to establish SRTP aborts streaming.

~ Assumption: DTLS handshake succeeds with peer identity.

- Exclusion: does not implement DRM.
- Exclusion: does not support unencrypted RTP.

> WebRTCCertificateIdentity
> WebRTCLibrarySelection