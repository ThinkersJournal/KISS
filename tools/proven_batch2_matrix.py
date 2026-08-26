"""#278 batch 2 -- thin wrapper over the generic proven_matrix.py core; record in conformance/PROVEN_BATCH2.md.

Only the ten mutation seeds and their clause map live here; the runner (DERIVED targets + baseline gate
+ --no-fail-fast + isolation matrix) is shared with batch 1 in proven_matrix.py. The two hardenings this
batch's driver introduced in #326 review -- the baseline gate and --no-fail-fast -- now live in the core,
so batch 1 inherits them too and the two drivers can no longer diverge on the runner.

Run from the batch worktree ROOT on a clean tree:  python tools/proven_batch2_matrix.py
"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from proven_matrix import run

CT = "conformance/src/contract.rs"; ST = "conformance/src/structural.rs"
SH = "conformance/src/shape_expr.rs"; DT = "conformance/src/determinism.rs"
AN = "conformance/src/announce.rs"; SK = "conformance/src/structure_key.rs"

BATCH = {  # proving test -> clause (ten CITED clauses migrated to PROVEN by this batch)
    "test_contract_text_field_encoding": "CONTRACT-6.11-0001",
    "test_contract_document_header_line": "CONTRACT-6.11-0002",
    "test_contract_document_field_order": "CONTRACT-6.11-0005",
    "test_contract_audited_status_derived": "CONTRACT-6.8-0008",
    "test_conform_per_output_comparator_selection": "CONFORM-6.8-0011",
    "test_shape_expr_serialization_golden": "OPS-6.20-0005",
    "test_ops_comparison_mask_is_selection": "OPS-6.0-0008",
    "test_synth_decline_framing": "SYNTH-6.6-0004",
    "test_synth_request_reuses_cyrq": "SYNTH-6.1-0002",
    "a1_elementwise_with_broadcast_operand": "CLASSIFY-6.7-0004",
}

# (proving_test, file, old, new) -- each `old` occurs exactly once (grep-verified). The KISC entry is a
# RAW byte string because the `\n` inside that Rust format-string literal is the two bytes backslash-n.
MUT = [
    ("test_contract_text_field_encoding", CT,
     b'format!("[{}]", elems.join(", "))', b'format!("[{}]", elems.join(","))'),
    ("test_contract_document_header_line", CT,
     br'"KISC {} {} len={} crc32={:08x}\n"', br'"XISC {} {} len={} crc32={:08x}\n"'),
    ("test_contract_document_field_order", CT,
     b'for (k, v) in fields {', b'for (k, v) in fields.iter().rev() {'),
    ("test_contract_audited_status_derived", CT,
     b'if g.declares_bounded_precision() {', b'if true || g.declares_bounded_precision() {'),
    ("test_conform_per_output_comparator_selection", ST,
     b'order_invariant_agree(actual, expected, abs_tol, rel_tol)',
     b'order_invariant_agree(actual, expected, 0.0, 0.0)'),
    ("test_shape_expr_serialization_golden", SH,
     b'ShapeExpr::SameAs { operand } => vec![TAG_SAME_AS, *operand],',
     b'ShapeExpr::SameAs { operand } => vec![*operand, TAG_SAME_AS],'),
    ("test_ops_comparison_mask_is_selection", DT,
     b'&["cmp_eq", "cmp_ne", "cmp_lt", "cmp_le", "cmp_gt", "cmp_ge"];',
     b'&["cmp_eq", "cmp_ne", "cmp_le", "cmp_gt", "cmp_ge"];'),
    ("test_synth_decline_framing", AN,
     b'b.extend_from_slice(&self.decline_code.to_le_bytes());',
     b'b.extend_from_slice(&(self.decline_code as u16).to_le_bytes());'),
    ("test_synth_request_reuses_cyrq", AN,
     b'b.extend_from_slice(&CYRQ.to_le_bytes());',
     b'b.extend_from_slice(&CYRQ.to_be_bytes());'),
    ("a1_elementwise_with_broadcast_operand", SK,
     b'.join(";")', b'.join(",")'),
]

if __name__ == "__main__":
    sys.exit(run(BATCH, MUT))
