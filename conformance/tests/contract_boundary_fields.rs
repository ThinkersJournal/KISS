// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract boundary fields (board 476): the optional Capabilities field
//! `requires_co_resident_launch` (KISS-CONTRACT-6.7-0007), the optional Guarantees field
//! `entry_exit_visibility` (KISS-CONTRACT-6.8-0014), and the rule that a contract states memory
//! visibility only at the launch boundary (KISS-CONTRACT-6.5-0020).
//!
//! Every arm drives the SHIPPED section readers in `kiss_conformance::contract` over the Appendix C
//! golden blocks, never a test-side parser.

use kiss_conformance::contract::{
    appendix_c_capabilities_block, appendix_c_guarantees_block, parse_capabilities_block, parse_guarantees_block,
    SectionDecline,
};

fn with_line(mut block: Vec<u8>, line: &str) -> Vec<u8> {
    block.extend_from_slice(line.as_bytes());
    block.push(b'\n');
    block
}

fn fields(r: Result<Vec<(String, String)>, SectionDecline>) -> Vec<String> {
    r.expect("the block parses").into_iter().map(|(k, _)| k).collect()
}

// Backs[enforced]: KISS-CONTRACT-6.7-0007 — the Capabilities section MAY carry `requires_co_resident_launch`, pinned last after `cost`, exactly `true` or `false`; absent means false; any other value, or a position before `cost`, is a typed decline.
#[test]
fn test_contract_requires_co_resident_launch_field() {
    let golden = appendix_c_capabilities_block();
    // absent: the golden block (no optional field) is accepted, so no existing contract is refused
    assert_eq!(fields(parse_capabilities_block(&golden)).last().map(String::as_str), Some("cost"));
    // present, last, `true` and `false` both accepted
    for v in ["true", "false"] {
        let b = with_line(golden.clone(), &format!("requires_co_resident_launch = {v}"));
        let f = fields(parse_capabilities_block(&b));
        assert_eq!(f.last().map(String::as_str), Some("requires_co_resident_launch"));
    }
    // any other spelling is a typed decline naming the field
    for bad in ["True", "1", "yes", "", "true "] {
        let b = with_line(golden.clone(), &format!("requires_co_resident_launch = {bad}"));
        assert_eq!(
            parse_capabilities_block(&b),
            Err(SectionDecline::BadFieldValue { field: "requires_co_resident_launch", got: bad.to_string() }),
            "6.7-0007: the value {bad:?} was not declined"
        );
    }
    // before `cost` it breaks the pinned order
    let text = String::from_utf8(golden.clone()).unwrap();
    let (head, cost) = text.rsplit_once("cost = ").unwrap();
    let early = format!("{head}requires_co_resident_launch = true\ncost = {cost}");
    assert_eq!(parse_capabilities_block(early.as_bytes()), Err(SectionDecline::WrongOrder));
}

// Backs[enforced]: KISS-CONTRACT-6.8-0014 — the Guarantees section MAY carry `entry_exit_visibility`, pinned last after `cost_provenance`, exactly `caller-ordered` or `visible-at-exit`; absent means `caller-ordered`; any other value is a typed decline.
#[test]
fn test_contract_entry_exit_visibility_field() {
    let golden = appendix_c_guarantees_block();
    assert_eq!(fields(parse_guarantees_block(&golden)).last().map(String::as_str), Some("cost_provenance"));
    for v in ["caller-ordered", "visible-at-exit"] {
        let b = with_line(golden.clone(), &format!("entry_exit_visibility = {v}"));
        assert_eq!(fields(parse_guarantees_block(&b)).last().map(String::as_str), Some("entry_exit_visibility"));
    }
    for bad in ["visible", "Caller-Ordered", "none", "", "visible-at-exit "] {
        let b = with_line(golden.clone(), &format!("entry_exit_visibility = {bad}"));
        assert_eq!(
            parse_guarantees_block(&b),
            Err(SectionDecline::BadFieldValue { field: "entry_exit_visibility", got: bad.to_string() }),
            "6.8-0014: the value {bad:?} was not declined"
        );
    }
}

// Backs[enforced]: KISS-CONTRACT-6.5-0020 — a contract states memory visibility only at the launch boundary; a field about memory only the kernel sees (scratch, shared, local, ordering, scheduling) is not in any section's schema and is declined as unknown.
#[test]
fn test_contract_visibility_boundary_only() {
    for name in ["scratch_visibility", "shared_memory_order", "memory_model", "scheduling", "barrier_scope"] {
        let c = with_line(appendix_c_capabilities_block(), &format!("{name} = x"));
        assert_eq!(parse_capabilities_block(&c), Err(SectionDecline::UnknownField(name.to_string())));
        let g = with_line(appendix_c_guarantees_block(), &format!("{name} = x"));
        assert_eq!(parse_guarantees_block(&g), Err(SectionDecline::UnknownField(name.to_string())));
    }
    // the boundary field itself is accepted: visibility is stated, but only here
    let g = with_line(appendix_c_guarantees_block(), "entry_exit_visibility = visible-at-exit");
    assert!(parse_guarantees_block(&g).is_ok());
}
