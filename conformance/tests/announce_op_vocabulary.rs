// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-Announce §7.2-0013: the handshake does not negotiate an op vocabulary. A reader carries an
//! `op_identity` it has never heard of without refusing anything; knowledge of the op is not an Announce matter.

use kiss_conformance::announce::*;

/// Enforces[enforced]: KISS-ANNOUNCE-7.2-0013 — an availability record, a contract query and the
/// negotiation all go through for an op the reader does not know.
/// TEETH: a reader that checked `op_identity` against an op registry (a vocabulary handshake by the
/// back door) would decline the unknown-op list or query below; a negotiation that needed an op list
/// would fail for the envelopes, which carry none.
#[test]
fn test_announce_op_vocabulary_not_negotiated() {
    // Names no registry holds: a provider-private spelling, and a name from a future vocabulary.
    for unknown in [&b"acme.rms_norm_residual_v9"[..], &b"zz_not_an_op"[..]] {
        let list = AvailabilityList {
            list_version: LIST_VERSION,
            records: vec![AvailabilityRecord { op_identity: unknown.to_vec(), structure_key: vec![0xAA, 0xBB], revision_hash: vec![0x42; 8] }],
        };
        assert_eq!(decode_availability_list(&list.encode()), Ok(list), "an unknown op is carried, never refused");

        let req = QueryRequest { identity: Identity { op_identity: unknown.to_vec(), structure_key: vec![0xAA, 0xBB], revision_hash: None, correlation: None } };
        assert_eq!(decode_query_request(&req.encode()), Ok(req), "a query naming an unknown op decodes");
    }

    // The negotiation is computed from the two envelopes alone, and those carry no op list: the same pair
    // negotiates identically whatever either side's op vocabulary, which is not an input at all.
    let (consumer, provider) = reference_negotiation_pair();
    // Presence of the bit is all a side says about Ops; set it on both and nothing more is exchanged.
    let speaks_ops = |e: &Envelope| Envelope { capabilities: e.capabilities | (1 << 50), ..e.clone() };
    let (c, p) = (speaks_ops(&consumer), speaks_ops(&provider));
    let (profile, live) = negotiate_live(&c, &p, true).expect("two sides that speak Ops negotiate with no op-set exchange");
    assert_eq!(live & (1 << 50), 1 << 50, "SPEAKS_OPS is live (both sides set it) and is the only Ops statement");
    assert_eq!(negotiate_live(&consumer, &provider, true).map(|(pr, _)| pr), Ok(profile), "the profile does not depend on Ops");
}
