// SPDX-License-Identifier: MIT OR Apache-2.0
//! KISS-CLASSIFY-6.6-0026 (`sk8`; CireSnave 2026-11-13, board 430 Q5 "b"): no key carries an escape for a type KISS
//! cannot name. A type no vocabulary names stays out of every key; the omission is reported in the Announce device
//! record (KISS-ANNOUNCE-6.6-0011), not spelled.

// Backs[enforced]: KISS-CLASSIFY-6.6-0026 — the Vulkan vocabulary spells no `x<n>` component type and omits a tuple that would contain an unnamed one; no published key token carries an `x<n>` component; the vocabulary version moved to 6; an `x<n>` in a key position is an unknown token.
#[test]
fn test_classify_unnamed_type_has_no_escape() {
    let vk = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/namespaces/vulkan.md")).unwrap();
    let norm = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let vk = norm(&vk);
    // the component-type list no longer ends in an `x<n>` alternative, and V-7 says the escape is gone.
    assert!(!vk.contains("or `x<n>` for a `VkComponentTypeKHR`"), "the x<n> alternative is removed from the component-type list");
    assert!(vk.contains("**There is no `x<n>` escape (removed at vocabulary version 6;"), "V-7 states the removal");
    assert!(vk.contains("is **omitted whole**"), "an unnamed type's tuple is omitted whole");
    assert!(vk.contains("**Vocabulary version:** 6"), "the Vulkan vocabulary version moved to 6");
    assert!(vk.contains("KISS-ANNOUNCE-6.6-0011"), "the omission is reported in the Announce device record");
    // the vocabulary version the vectors record is the same 6.
    assert!(kiss_conformance::reference_vectors::NAMESPACE_VOCAB_VERSIONS.contains(&("vulkan", 6, 5)));
    // no published key (positive or derived) spells an `x<digits>` component.
    for file in ["structure_key_vectors.json", "structure_key_derivation_vectors.json"] {
        let text = std::fs::read_to_string(format!("{}/corpus/{file}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let bytes = text.as_bytes();
        for i in 1..bytes.len().saturating_sub(2) {
            if bytes[i] == b'x' && bytes[i - 1] == b'-' && bytes[i + 1].is_ascii_digit() {
                panic!("{file}: an x<n> component at byte {i}: {}", &text[i.saturating_sub(20)..(i + 12).min(text.len())]);
            }
        }
    }
    // an `x<n>` in a dtype position is an unknown token, exactly as any unrecognized spelling.
    let tok = "sk8|ew2|x1000491002|cuda:sm89|ix32|grid|r2|co/00/v4/d16/f/x1000491002|-";
    assert_eq!(kiss_conformance::structure_key::from_token(tok), Err(kiss_conformance::structure_key::KeyDecline::UnknownDtype));
    let spec = norm(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../spec/classify.md")).unwrap());
    assert!(spec.contains("**KISS-CLASSIFY-6.6-0026** — **From `sk8`, no `structure_key` and no target token carries an escape for a type KISS cannot name.**"));
}
