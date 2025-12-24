#Spell: WebRTCCertificateIdentity
^ Intent: provision DTLS identity for each peer connection safely and reproducibly.

@ Identity
  : () -> (certificate, private_key, fingerprint)
  ! Guarantee: each PeerConnection has a DTLS certificate and matching private key.
  ! Guarantee: certificate fingerprint is exposed to SDP generation.
  ! Guarantee: private key never leaves process memory by default.
  ~ Assumption: webrtc-rs library handles certificate generation securely.
  - Exclusion: does not implement custom certificate management beyond library defaults.
  - Exclusion: does not expose private keys externally.
  > WebRTCLibrarySelection
