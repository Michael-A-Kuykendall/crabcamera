# WebRTC Spellbook Architecture Prompt

## Mission
Design a comprehensive Sorcery spellbook for implementing real WebRTC streaming functionality in CrabCamera. The spellbook must enable flawless implementation of production-ready WebRTC peer-to-peer video/audio streaming while maintaining CrabCamera's architectural constraints.

## Context: CrabCamera Project Overview

### Project Identity
- **Name:** CrabCamera
- **Mission:** Invisible camera infrastructure - drop it in, it works
- **License:** MIT (no GPL contamination)
- **Philosophy:** Pure Rust, zero FFmpeg, single binary
- **Platform:** Cross-platform desktop (Windows, macOS, Linux) Tauri plugin

### Current Architecture
```
CrabCamera (Tauri Plugin)
├── nokhwa (camera capture)
├── openh264 (video encoding) [H.264]
├── libopus_sys (audio encoding) [Opus]
├── cpal (audio capture)
└── muxide (MP4 muxing, private path dependency)
```

### Existing WebRTC State
- **Location:** `src/webrtc/` (mod.rs, peer.rs, streaming.rs)
- **Current Status:** Mock implementations with experimental warnings
- **Mock Components:**
  - SDP: Template-based offer/answer generation
  - ICE: Hardcoded placeholder candidates
  - Encoding: Zero-filled buffers instead of real H.264/Opus frames
  - Data Channels: No-op send operations
- **API Structure:** Complete but non-functional

### Sorcery Doctrine (Reference)
- **#Spell:** Identifier
- **^ Intent:** 1-2 lines bounded outcome
- **@** Entity/component
- **:** Contract shape (input → output)
- **!** Guarantee/invariant
- **~** Assumption
- **-** Explicit exclusion
- **>** Dependency
- **?** Open question (blocks sealing)

### Sealing Rules
- Intent exists
- No open questions
- Guarantees explicit
- At least one exclusion
- Exactly one thing per spell

## Requirements: Real WebRTC Implementation

### Functional Requirements
1. **Peer Connection Management**
   - Real SDP offer/answer exchange
   - Actual ICE candidate gathering and connectivity checks
   - DTLS handshake for secure transport
   - SRTP encryption for media streams

2. **Media Streaming**
   - RTP packetization of H.264 video frames
   - RTP packetization of Opus audio frames
   - Real-time frame capture from camera/microphone
   - Integration with existing openh264/libopus encoders

3. **Data Channels**
   - Reliable/unreliable data transmission
   - Binary/text message support
   - Channel state management

4. **Streaming Controls**
   - Start/stop streaming
   - Dynamic bitrate/fps/resolution adjustment
   - Subscriber management (multiple peers)
   - Stream statistics and monitoring

### Non-Functional Requirements
- **Performance:** Real-time streaming (30fps @ 720p target)
- **Reliability:** Robust error handling and recovery
- **Compatibility:** WebRTC 1.0 standard compliant
- **Security:** DTLS-SRTP encryption mandatory

## Constraints & Boundaries

### Hard Constraints
- **Pure Rust:** No C/C++ bindings except existing (libopus_sys, openh264)
- **No FFmpeg:** Never add FFmpeg, GStreamer, or libav
- **Licensing:** Only MIT/Apache-2.0 compatible dependencies
- **Single Binary:** No external runtime dependencies
- **Existing Integration:** Must work with current camera capture (nokhwa) and audio capture (cpal)

### Architectural Assumptions
- **~** WebRTC library: Use webrtc-rs (pure Rust, Apache-2.0) or implement core protocols manually
- **~** Network: Assume NAT traversal works via STUN/TURN
- **~** Hardware: Assume sufficient CPU/GPU for real-time encoding
- **~** Browser Compatibility: Target modern WebRTC-compatible browsers

### Explicit Exclusions
- **-** No WebRTC server (SFU/MCU) - peer-to-peer only
- **-** No simulcast/SVC - single stream per peer
- **-** No custom codecs - use existing H.264/Opus
- **-** No recording integration - streaming only
- **-** No advanced features (simulcast, data channel priority, etc.)

## Input/Output Specifications

### Inputs (What the Spell Receives)
1. **Camera Frames:** Raw YUV/RGB frames from nokhwa capture
2. **Audio Samples:** PCM audio from cpal capture
3. **Peer Commands:** Connection requests, SDP from remote peers
4. **Stream Config:** Bitrate, resolution, fps parameters
5. **Network Events:** ICE candidates, connection state changes

### Outputs (What the Spell Produces)
1. **Encoded RTP Packets:** H.264 video in RTP format
2. **Encoded RTP Packets:** Opus audio in RTP format
3. **SDP Offers/Answers:** Valid WebRTC session descriptions
4. **ICE Candidates:** Real network candidates for connectivity
5. **Data Messages:** Binary/text data over WebRTC data channels
6. **Stream Statistics:** Bitrate, fps, latency, packet loss metrics

### Data Flow Architecture
```
Camera/Microphone → Encoder → RTP Packetizer → SRTP Encrypt → Network
                                      ↓
Remote Peer ← SDP Exchange ← ICE Negotiation ← DTLS Handshake
```

## Spellbook Structure Requirements

### Required Spells (Minimum)
1. **#WebRTCRealSDP** - Real SDP generation and parsing
2. **#WebRTCRealICE** - ICE candidate gathering and connectivity
3. **#WebRTCRealEncoding** - Frame encoding integration
4. **#WebRTCRTP** - RTP/SRTP packetization and transmission
5. **#WebRTCPeerIntegration** - Complete peer connection lifecycle
6. **#WebRTCDataChannels** - Data channel implementation
7. **#WebRTCStreamingControls** - Stream management and statistics

### Spell Dependencies (DAG)
```
#WebRTCRealSDP → #WebRTCPeerIntegration
#WebRTCRealICE → #WebRTCPeerIntegration  
#WebRTCRealEncoding → #WebRTCRTP
#WebRTCRTP → #WebRTCPeerIntegration
#WebRTCDataChannels → #WebRTCPeerIntegration
#WebRTCStreamingControls → #WebRTCPeerIntegration
```

### Testing Integration
- Each spell must include test specifications
- Integration tests for end-to-end streaming
- Mock-free implementation (replace all current mocks)

## Deliverable Format

### Spellbook File Structure
```
docs/WEBRTC_SPELLBOOK.md
├── Spell declarations with full glyph notation
├── Implementation contracts (input/output)
├── Dependency chains
├── Test specifications
├── Integration points with existing codebase
└── Sealing verification checklist
```

### Spell Template Format
```
#SpellName
^ Intent: Single bounded outcome in 1-2 lines

@ Component1 : input_type → output_type
@ Component2 : input_type → output_type

! Guarantee1: Specific invariant
! Guarantee2: Specific invariant

~ Assumption1: Required condition
~ Assumption2: Required condition

- Exclusion1: What is explicitly not included
- Exclusion2: What is explicitly not included

> DependencySpell1
> DependencySpell2

[Implementation Notes]
[Test Cases]
```

## Success Criteria

### Spell Quality
- **Sealed:** No open questions, all guarantees explicit, exclusions present
- **Bounded:** Each spell does exactly one thing
- **Composable:** Clear dependency chains
- **Testable:** Verifiable contracts and invariants

### Architecture Quality
- **Maintains Philosophy:** Pure Rust, zero FFmpeg, single binary
- **Integrates Cleanly:** Works with existing capture/encoding pipeline
- **Production Ready:** Handles errors, recovers from failures
- **Performance Conscious:** Real-time streaming capabilities

### Implementation Readiness
- **Dependency Analysis:** Clear what new crates/libraries needed
- **Migration Path:** How to transition from mock to real
- **Risk Assessment:** Potential integration challenges identified

## Open Questions (Resolve Before Sealing)
? Which WebRTC Rust library? (webrtc-rs vs manual implementation)
? DTLS certificate management approach?
? TURN server integration (if needed)?
? Browser compatibility testing strategy?
? Performance profiling for real-time encoding?

---

**Final Output:** A complete `docs/WEBRTC_SPELLBOOK.md` file with all spells properly sealed, ready for implementation invocation.</content>
<parameter name="filePath">c:\Users\micha\repos\crabcamera\docs\WEBRTC_SPELL_PROMPT.md