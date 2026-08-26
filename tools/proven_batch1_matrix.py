"""#278 batch 1 -- thin wrapper over the generic proven_matrix.py core; record in conformance/PROVEN_BATCH1.md.

Only the ten mutation seeds and their clause map live here; the runner (DERIVED targets + baseline gate
+ --no-fail-fast + isolation matrix) is shared with batch 2 in proven_matrix.py. The baseline gate and
--no-fail-fast that the #326 review found missing from this file are now inherited from the core BY
CONSTRUCTION -- which is why the harmonize marker that used to sit here (#327) is gone: batch 1 and
batch 2 can no longer diverge on the runner, because there is only one.

Run from the batch worktree ROOT on a clean tree:  python tools/proven_batch1_matrix.py
"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from proven_matrix import run

SK = "conformance/src/structure_key.rs"; IT = "conformance/src/integer.rs"
ST = "conformance/src/structural.rs"; CT = "conformance/src/contract.rs"

BATCH = {  # proving test -> clause (the ten demonstrated; 6.1-0008 migrated in, see PROVEN_BATCH1.md)
    "reject_unknown_dtype": "CLASSIFY-6.1-0001", "reject_unknown_op_family": "CLASSIFY-6.5-0006",
    "reject_wrong_field_count": "CLASSIFY-6.7-0009", "test_classify_work_class_element_count": "CLASSIFY-6.5-0007",
    "test_ops_add_sub_mul_wrapping": "OPS-6.4-0001", "test_ops_neg_integer": "OPS-6.4-0003",
    "scan_is_length_preserving": "OPS-6.11-0003", "scatter_oob_writes_are_skipped": "OPS-6.11-0005",
    "scatter_atomic_max_min_nan_propagating": "OPS-6.11-0010", "test_contract_version_value_pinned": "CONTRACT-6.1-0008"}

# (proving_test, file, old, new) -- each `old` occurs exactly once; the mutation attacks the clause's
# obligation. Byte strings so the seed/restore is line-ending exact.
MUT = [
    ("reject_unknown_dtype", SK, b"if !DTYPES.contains(&f[2]) {", b"if false && !DTYPES.contains(&f[2]) {"),
    ("reject_unknown_op_family", SK, b"if !OP_FAMILIES.contains(&f[1]) {", b"if false && !OP_FAMILIES.contains(&f[1]) {"),
    ("reject_wrong_field_count", SK, b"if f.len() != 9 && f.len() != 10 {", b"if false && f.len() != 9 && f.len() != 10 {"),
    ("test_classify_work_class_element_count", SK, b"c if c <= 1024 => WorkClass::Block,", b"c if c <= 1024 => WorkClass::Grid,"),
    ("test_ops_add_sub_mul_wrapping", IT, b"fn add(self, rhs: Self) -> Self { self.wrapping_add(rhs) }", b"fn add(self, rhs: Self) -> Self { self.saturating_add(rhs) }"),
    ("test_ops_neg_integer", IT, b"fn neg(self) -> Self { self.wrapping_neg() }", b"fn neg(self) -> Self { !self }"),
    ("scan_is_length_preserving", ST, b"    }\n    out\n}", b"    }\n    out.pop();\n    out\n}"),
    ("scatter_oob_writes_are_skipped", ST, b"continue; // \xc2\xa76.11-0005: OOB writes are skipped.", b"(); // \xc2\xa76.11-0005 skip REMOVED (mutation)"),
    ("scatter_atomic_max_min_nan_propagating", ST, b"Combine::AtomicMax => max_prop(dest[i], s),", b"Combine::AtomicMax => dest[i].max(s),"),
    ("test_contract_version_value_pinned", CT, b'if version != "1" {', b'if version != "2" {'),
]

if __name__ == "__main__":
    sys.exit(run(BATCH, MUT))
