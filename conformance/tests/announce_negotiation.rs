// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Announce §7.3-0003: the live capability word of a negotiated pair is role-aware. The golden pair
//! is the clause's worked vector; the plain AND is the trap it exists to rule out.

use kiss_conformance::announce::*;

/// KISS-ANNOUNCE-7.3-0003.
#[test]
fn test_announce_live_capabilities_role_aware() {
    // Clause: KISS-ANNOUNCE-7.3-0003.
    let (consumer, provider) = reference_negotiation_pair();
    let golden = 0x0011_0003_0000_0001u64;

    // Consumer local, provider remote.
    assert_eq!(negotiate_live(&consumer, &provider, true), Ok((1, golden)));
    // Roles exchanged: the same word, whichever side computes it.
    assert_eq!(negotiate_live(&provider, &consumer, false), Ok((1, golden)));

    // The trap: a plain AND loses both provider-only FEAT bits (PROVISION_ON_REQUEST, CONTRACT_QUERY).
    let plain = consumer.capabilities & provider.capabilities;
    assert_eq!(plain, 0x0011_0000_0000_0001);
    assert_ne!(plain, golden);
    assert_eq!(golden & FEAT_MASK, 0x0000_0003_0000_0000, "both FEAT bits are live, from the provider's word");

    // Only the provider's word decides a FEAT bit: a consumer that sets one does not make it live.
    let liar = Envelope { capabilities: consumer.capabilities | (1 << 33), ..consumer.clone() };
    let silent_provider = Envelope { capabilities: provider.capabilities & !(1 << 33), ..provider.clone() };
    let (_, live) = negotiate_live(&liar, &silent_provider, true).unwrap();
    assert_eq!(live & (1 << 33), 0);

    // An EXT or SUB bit is live only if both set it: bit 53 (consumer only) and bit 55 (provider only) are off.
    assert_eq!(golden & ((1 << 53) | (1 << 55) | (1 << 1)), 0);
    // Masking is by axis, not recognition: a bit in no assigned range is treated like any other bit of its axis.
    assert_eq!(live_capabilities(1 << 20, 1 << 20, true), 1 << 20);
    assert_eq!(live_capabilities(1 << 40, 0, true), 0);
    assert_eq!(live_capabilities(0, 1 << 40, true), 1 << 40);
}
