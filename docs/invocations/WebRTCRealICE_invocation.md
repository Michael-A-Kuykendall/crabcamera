#Spell: WebRTCRealICE

^ Intent: implement real ICE agent lifecycle and connectivity establishment.

@ RealICE
  : config -> connected_state
  ! Guarantee: ICE agent is properly initialized with configuration.
  ! Guarantee: ICE gathering completes successfully.
  ! Guarantee: connectivity checks pass for valid network paths.
  ! Guarantee: connection state transitions are handled correctly.
  ! Guarantee: RTCConfiguration includes ICE servers for gathering.
  ! Guarantee: peer connection handles ICE state machine internally.
  ! Guarantee: connection state can be queried via get_connection_state.
  ~ Assumption: network allows ICE connectivity (no symmetric NAT issues).
  - Exclusion: does not implement ICE protocol itself.
  - Exclusion: does not provide STUN/TURN servers.
  > WebRTCICECandidateExchange
