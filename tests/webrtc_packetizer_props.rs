//! Property-based tests for WebRTC RTP packetizers.
//!
//! Focus: stable invariants (MTU bounds, marker semantics, sequencing, and
//! lossless reconstruction for packetization).

#![cfg(feature = "webrtc")]

use proptest::prelude::*;

use crabcamera::webrtc::streaming::{H264RTPPacketizer, OpusRTPPacketizer, RtpPayload};

fn build_annex_b_access_unit(nals: &[Vec<u8>]) -> Vec<u8> {
    let mut au = Vec::new();
    for nal in nals {
        au.extend_from_slice(&[0, 0, 0, 1]);
        au.extend_from_slice(nal);
    }
    au
}

fn reconstruct_nals_from_payloads(payloads: &[RtpPayload]) -> Result<Vec<Vec<u8>>, String> {
    let mut nals: Vec<Vec<u8>> = Vec::new();
    let mut current_fu: Option<Vec<u8>> = None;

    for p in payloads {
        if p.data.is_empty() {
            return Err("Empty RTP payload data".to_string());
        }

        let nal_type = p.data[0] & 0x1F;

        if nal_type == 28 {
            // FU-A
            if p.data.len() < 3 {
                return Err("FU-A payload too short".to_string());
            }

            let fu_indicator = p.data[0];
            let fu_header = p.data[1];
            let start = (fu_header & 0x80) != 0;
            let end = (fu_header & 0x40) != 0;

            let orig_nal_header = (fu_indicator & 0xE0) | (fu_header & 0x1F);
            let fragment = &p.data[2..];

            if start {
                if current_fu.is_some() {
                    return Err("FU-A start while previous FU incomplete".to_string());
                }
                let mut nal = Vec::with_capacity(1 + fragment.len());
                nal.push(orig_nal_header);
                nal.extend_from_slice(fragment);
                current_fu = Some(nal);
            } else {
                let Some(ref mut nal) = current_fu else {
                    return Err("FU-A continuation without start".to_string());
                };
                nal.extend_from_slice(fragment);
            }

            if end {
                let nal = current_fu.take().ok_or("FU-A end without start".to_string())?;
                nals.push(nal);
            }
        } else {
            // Single NAL
            if current_fu.is_some() {
                return Err("Single NAL while FU-A in progress".to_string());
            }
            nals.push(p.data.clone());
        }
    }

    if current_fu.is_some() {
        return Err("FU-A did not terminate with end bit".to_string());
    }

    Ok(nals)
}

fn nal_unit_strategy() -> impl Strategy<Value = Vec<u8>> {
    // Restrict to "single NAL" types (1..=23) to keep generation valid.
    // Use NRI=3 (0x60) and F=0.
    (1u8..=23u8, proptest::collection::vec(any::<u8>(), 0..5000)).prop_map(|(nal_type, mut rest)| {
        let mut nal = Vec::with_capacity(rest.len() + 1);
        nal.push(0x60 | nal_type);
        nal.append(&mut rest);
        nal
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// INVARIANT: H264 packetization respects MTU budget, timestamps are stable,
    /// sequence numbers are contiguous, marker bit is set exactly once and only
    /// on the final packet of the access unit.
    #[test]
    fn h264_packetize_invariants_hold(
        mtu in 200usize..1500usize,
        timestamp in any::<u64>(),
        nals in proptest::collection::vec(nal_unit_strategy(), 1..6),
    ) {
        let effective_mtu = mtu.min(1200);
        let access_unit = build_annex_b_access_unit(&nals);

        let mut packetizer = H264RTPPacketizer::new(mtu);
        let payloads = packetizer.packetize(&access_unit, timestamp)
            .expect("packetize should succeed for valid Annex B AU");

        prop_assert!(!payloads.is_empty());

        // MTU invariant (+ internal budget: payload only, header is accounted for by the packetizer)
        for p in &payloads {
            prop_assert!(!p.data.is_empty());
            prop_assert!(p.data.len() <= effective_mtu - 12);
            prop_assert_eq!(p.timestamp, timestamp);
        }

        // Sequence numbers contiguous from 0 for a fresh packetizer.
        for (i, p) in payloads.iter().enumerate() {
            prop_assert_eq!(p.sequence_number, i as u16);
        }

        // Marker semantics: exactly one marker, and it's the last packet.
        let marker_count = payloads.iter().filter(|p| p.marker).count();
        prop_assert_eq!(marker_count, 1);
        prop_assert!(payloads.last().unwrap().marker);

        // Contract: lossless reconstruction of NAL units.
        let reconstructed = reconstruct_nals_from_payloads(&payloads)
            .map_err(|e| TestCaseError::fail(e))?;
        prop_assert_eq!(reconstructed, nals);
    }

    /// CONTRACT: invalid data that doesn't start with an Annex B start code must error.
    #[test]
    fn h264_packetize_rejects_non_annex_b(
        timestamp in any::<u64>(),
        data in proptest::collection::vec(any::<u8>(), 0..256)
            .prop_filter("must not begin with AnnexB start code", |v| {
                !(v.starts_with(&[0,0,1]) || v.starts_with(&[0,0,0,1]))
            }),
    ) {
        let mut packetizer = H264RTPPacketizer::new(1200);
        let result = packetizer.packetize(&data, timestamp);
        prop_assert!(result.is_err());
    }

    /// INVARIANT: Opus RTP packetizer increments sequence and timestamp monotonically.
    #[test]
    fn opus_packetizer_invariants_hold(
        p1 in proptest::collection::vec(any::<u8>(), 0..2000),
        p2 in proptest::collection::vec(any::<u8>(), 0..2000),
        p3 in proptest::collection::vec(any::<u8>(), 0..2000),
        s1 in 1u32..5000u32,
        s2 in 1u32..5000u32,
        s3 in 1u32..5000u32,
    ) {
        let mut packetizer = OpusRTPPacketizer::new();

        let a = packetizer.packetize(&p1, s1).expect("packetize");
        let b = packetizer.packetize(&p2, s2).expect("packetize");
        let c = packetizer.packetize(&p3, s3).expect("packetize");

        prop_assert_eq!(a.sequence_number, 0);
        prop_assert_eq!(b.sequence_number, 1);
        prop_assert_eq!(c.sequence_number, 2);

        prop_assert!(a.marker);
        prop_assert!(b.marker);
        prop_assert!(c.marker);

        prop_assert_eq!(a.timestamp, 0);
        prop_assert_eq!(b.timestamp, s1 as u64);
        prop_assert_eq!(c.timestamp, (s1 + s2) as u64);

        prop_assert_eq!(a.data, p1);
        prop_assert_eq!(b.data, p2);
        prop_assert_eq!(c.data, p3);
    }
}
