#Spell: WebRTCICECandidateExchange

^ Intent: implement real ICE candidate gathering and exchange for connectivity establishment.

@ ICECandidateExchange
  : (local_candidates, remote_candidates) -> connectivity
  ! Guarantee: local ICE candidates are gathered from available network interfaces.
  ! Guarantee: remote ICE candidates are processed and added to connection.
  ! Guarantee: connectivity checks succeed when valid candidates are exchanged.
  ! Guarantee: ICE state transitions are handled correctly.
  ! Guarantee: PeerConnection::add_ice_candidate processes remote candidates.
  ! Guarantee: ICE servers from config are used for gathering.
  ! Guarantee: local candidates are collected via event handling.
  ~ Assumption: STUN/TURN servers are available for NAT traversal.
  - Exclusion: does not implement STUN/TURN server.
  - Exclusion: does not handle network configuration beyond candidate gathering.
  > WebRTCRealSDP
