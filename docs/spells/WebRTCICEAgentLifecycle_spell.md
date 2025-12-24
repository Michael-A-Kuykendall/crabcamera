#Spell: WebRTCICEAgentLifecycle
^ Intent: manage ICE agent state transitions and event handling for robust connection establishment.

@ ICEState
  : event -> new_state

! Guarantee: connection state transitions are observable and deterministic.
! Guarantee: timeouts and retries have bounded behavior.

~ Assumption: underlying ICE library emits state events.

- Exclusion: does not implement candidate parsing.
- Exclusion: does not implement networking sockets directly.

> WebRTCRealICE
