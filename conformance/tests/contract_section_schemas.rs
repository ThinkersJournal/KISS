//! Byte-level field-schema tests for the Interface (§6.5-0001), Capabilities (§6.7-0001) and
//! Provenance (§6.9-0001) sections — the three whose field-line schema was verified only as PROSE.
//!
//! The census (architect-allocated) measured that these three sections' §9-named tests are
//! spec-text (`read_spec`, checking the clause lists its fields) and that NO test ever builds one
//! of these blocks and parses it — their byte form fell through because the golden renders 3 of 7
//! blocks (#365). This is the #489 pattern generalized (Dispatch was the 4th such section).
//!
//! ⚠️ These tests keep the spec-text tests (they verify the schema is STATED); these verify it is
//! ENFORCED — genuinely different checks, both valid. They parse with the SHIPPED per-section
//! readers in `kiss_conformance::contract`, never a test-only parser (a test-side parser agreeing
//! with a test-side emitter proves only that one author wrote both — the census defect one level up).
//!
//! The arms per section (#489's template, extended):
//!   round-trip   build the block, parse it back, assert the fields round-trip;
//!   byte-pin     the emitted block equals the exact expected bytes (in each per-section test);
//!   decline      a missing field / a wrong field ORDER / an unknown field / a MALFORMED line
//!                (no ` = `) / a mismatched section HEADING each decline;
//!   discriminate the section's OWN MUST-NOT field — the plausible wrong impl each clause forbids.

use kiss_conformance::contract::{
    parse_capabilities_block, parse_interface_block, parse_provenance_block, render_block,
    SectionDecline, Value,
};

/// Build a section block from `(key, value)` pairs, in order.
fn block(id: u8, name: &str, fields: &[(&str, &str)]) -> Vec<u8> {
    let vals: Vec<(&str, Value)> =
        fields.iter().map(|(k, v)| (*k, Value::Str((*v).to_string()))).collect();
    render_block(id, name, &vals)
}

/// The shared signature of the per-section readers (§6.5/6.7/6.9-0001).
type SectionParser = fn(&[u8]) -> Result<Vec<(String, String)>, SectionDecline>;

/// The FIELD-SCHEMA arms shared across the flat-field-schema sections: round-trip, plus the
/// missing / wrong-order / unknown / forbidden declines. `parse` is the SHIPPED per-section reader;
/// `forbidden` is the section's own MUST-NOT field (the discriminate arm). Block-structure declines
/// (a malformed line, a bad heading) are asserted by [`assert_structural_declines`], called at the end.
fn assert_schema_arms(
    parse: SectionParser,
    id: u8,
    name: &str,
    fields: &[(&str, &str)],
    forbidden: (&str, &str),
) {
    // round-trip — a valid block parses back to exactly its fields, in order.
    let want: Vec<(String, String)> =
        fields.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect();
    assert_eq!(parse(&block(id, name, fields)), Ok(want), "{name}: round-trip");

    // decline — a missing required field (drop the last).
    let mut miss = fields.to_vec();
    let dropped = miss.pop().unwrap().0;
    assert_eq!(
        parse(&block(id, name, &miss)),
        Err(SectionDecline::MissingField(dropped.to_string())),
        "{name}: a missing field must decline"
    );

    // decline — a wrong field ORDER (swap the first two).
    let mut reordered = fields.to_vec();
    reordered.swap(0, 1);
    assert_eq!(
        parse(&block(id, name, &reordered)),
        Err(SectionDecline::WrongOrder),
        "{name}: a reordered schema must decline"
    );

    // decline — an unknown field.
    let mut unknown = fields.to_vec();
    unknown.push(("bogus_field", "x"));
    assert_eq!(
        parse(&block(id, name, &unknown)),
        Err(SectionDecline::UnknownField("bogus_field".to_string())),
        "{name}: an unknown field must decline"
    );

    // DISCRIMINATE — the section's own MUST-NOT field: a plausible wrong impl, not a strawman.
    let mut forb = fields.to_vec();
    forb.push(forbidden);
    assert_eq!(
        parse(&block(id, name, &forb)),
        Err(SectionDecline::ForbiddenField(forbidden.0.to_string())),
        "{name}: the forbidden field {:?} must decline",
        forbidden.0
    );

    // block-structure declines (a malformed line, a bad heading) — asserted in the helper below.
    assert_structural_declines(parse, id, name, fields);
}

/// The BLOCK-STRUCTURE decline arms, shared with [`assert_schema_arms`]: a field line without a
/// ` = ` separator (§6.11-0001) and a mismatched section heading. Split out so each helper keeps a
/// single, legible responsibility (and stays under the file's per-function length limit).
fn assert_structural_declines(parse: SectionParser, id: u8, name: &str, fields: &[(&str, &str)]) {
    // decline — a MALFORMED field line (no ` = ` separator), §6.11-0001. Corrupt the first field
    // line of an otherwise-valid block: the ONLY difference from the round-trip arm is the removed
    // separator, so a parser that failed to detect it would return Ok and this exact-payload
    // assertion would fail — the arm discriminates, it does not merely pass.
    let malformed = String::from_utf8(block(id, name, fields))
        .expect("block is valid utf-8")
        .replacen(" = ", " ", 1) // the first ` = ` is the first field line's separator
        .into_bytes();
    assert_eq!(
        parse(&malformed),
        Err(SectionDecline::MalformedLine(format!("{} {}", fields[0].0, fields[0].1))),
        "{name}: a field line without ` = ` must decline"
    );

    // decline — a mismatched section HEADING: the right fields under the WRONG section number
    // (`[section:<id+1>:<name>]` fed to a reader that expects `<id>`), the section-transposition
    // mistake a heading-blind parser would accept.
    assert_eq!(
        parse(&block(id.wrapping_add(1), name, fields)),
        Err(SectionDecline::BadHeading),
        "{name}: a mismatched section heading must decline"
    );
}

/// Byte-form coverage for §6.5-0001 (its §9-named backing stays the spec-text
/// `test_contract_interface_field_schema`, which verifies the schema is STATED; this exercises that
/// it is ENFORCED on bytes) — the Interface section carries exactly its seven fields in order;
/// discriminate: an independent `target` field is forbidden (the target is the Identity
/// `target_capability`, §6.3-0007 / §6.5-0003) — the "the interface names its target" mistake.
#[test]
fn test_contract_interface_block_bytes() {
    let fields: [(&str, &str); 7] = [
        ("entry_point", "k"),
        ("rank", "2"),
        ("positional_signature", "[]"),
        ("launch_scalars", "[]"),
        ("count_unit", "elements"),
        ("in_place", "false"),
        ("alignment_bytes", "1"),
    ];
    assert_schema_arms(parse_interface_block, 3, "interface", &fields, ("target", "cuda:sm89"));
    // byte-pin — the exact serialized block (§6.11-0004 heading + §6.11-0001 field lines).
    assert_eq!(
        block(3, "interface", &fields),
        b"[section:3:interface]\nentry_point = k\nrank = 2\npositional_signature = []\n\
          launch_scalars = []\ncount_unit = elements\nin_place = false\nalignment_bytes = 1\n"
            .to_vec(),
        "KISS-CONTRACT-6.5-0001: Interface block bytes"
    );
}

/// Byte-form coverage for §6.7-0001 (its §9-named backing stays the spec-text
/// `test_contract_capabilities_field_schema`; this exercises enforcement on bytes) — the
/// Capabilities section carries exactly its eight fields in order; discriminate: an Interface-only
/// field (`count_unit`/`in_place`/`alignment_bytes`) leaking in is forbidden (§6.5-0001) — the
/// cross-section leak a flat unscoped parser would allow.
#[test]
fn test_contract_capabilities_block_bytes() {
    let fields: [(&str, &str); 8] = [
        ("accept_predicate", "sk4|bin|f32"),
        ("supported_dtype_set", "[f32]"),
        ("awkward_layout_strategy", "decline"),
        ("in_place_eligible_variants", "[]"),
        ("index_width", "ix32"),
        ("determinism_class", "exact-byte"),
        ("precision_class", "strict"),
        ("cost", "1"),
    ];
    assert_schema_arms(parse_capabilities_block, 5, "capabilities", &fields, ("count_unit", "elements"));
    assert_eq!(
        block(5, "capabilities", &fields),
        b"[section:5:capabilities]\naccept_predicate = sk4|bin|f32\nsupported_dtype_set = [f32]\n\
          awkward_layout_strategy = decline\nin_place_eligible_variants = []\nindex_width = ix32\n\
          determinism_class = exact-byte\nprecision_class = strict\ncost = 1\n"
            .to_vec(),
        "KISS-CONTRACT-6.7-0001: Capabilities block bytes"
    );
}

/// Byte-form coverage for §6.9-0001 (its §9-named backing stays the spec-text
/// `test_contract_provenance_field_schema`; this exercises enforcement on bytes) — the Provenance
/// section carries exactly its five fields in order; discriminate: `audited_status` is forbidden (a
/// derived Guarantees field, §6.8-0001/-0008) — the "audited_status is a trust field, put it in
/// Provenance" mistake.
#[test]
fn test_contract_provenance_block_bytes() {
    let fields: [(&str, &str); 5] = [
        ("kernel_source", "authored"),
        ("revision_base", "r0"),
        ("revision_hash", "abc"),
        ("cost_provenance", "measured"),
        ("negotiation_metadata", "0:"),
    ];
    assert_schema_arms(parse_provenance_block, 7, "provenance", &fields, ("audited_status", "audited"));
    assert_eq!(
        block(7, "provenance", &fields),
        b"[section:7:provenance]\nkernel_source = authored\nrevision_base = r0\n\
          revision_hash = abc\ncost_provenance = measured\nnegotiation_metadata = 0:\n"
            .to_vec(),
        "KISS-CONTRACT-6.9-0001: Provenance block bytes"
    );
}
