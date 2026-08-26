"""#278 batch 2 — the re-runnable isolation-matrix driver behind conformance/PROVEN_BATCH2.md.

Same contract as tools/proven_batch1_matrix.py: for each batch clause it seeds a one-site
mutation of the clause's IMPLEMENTATION (the obligation subject, convention 15) in
conformance/src/, runs the relevant test binaries UNFILTERED, records which tests fail, and
restores the source BYTE-FOR-BYTE (try/finally on every path, seed-applied + byte-exact-restore
asserted). Prints per-mutation kills, the isolation matrix (each seed must kill EXACTLY ONE batch
test = its own), and the demonstration defect rate.

Two differences from batch 1, both because this batch's proving tests span BOTH integration tests
(conformance/tests/*.rs) AND lib unit tests (#[cfg(test)] in conformance/src/*.rs):
  * run_cargo passes `--lib` (for test_conform_per_output_comparator_selection and
    test_ops_comparison_mask_is_selection, which live in src/per_output.rs) in addition to the
    five integration binaries that hold the other eight proving tests;
  * a FAILED line for a lib unit test is module-qualified (`per_output::tests::NAME`), so failed
    names are matched by their LAST `::` component against the (bare) BATCH keys.

This driver MUTATES source while it runs. Run from the batch worktree ROOT on a CLEAN tree:
    python tools/proven_batch2_matrix.py
WARNING: an interrupt mid-cargo leaves the current seed applied (the finally restores it, but
kill -9 does not). `git diff conformance/src` must be empty after a clean run; if a run was
killed, `git checkout -- conformance/src` restores it (this worktree carries no src/ edits).
"""
import subprocess, re, sys

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

# (proving_test, file, old, new) — each `old` occurs exactly once (grep-verified); the mutation
# attacks the clause's obligation. Byte strings so seed/restore is line-ending exact; the KISC
# entry is a RAW byte string because the `\n` inside that Rust format-string literal is the two
# bytes backslash-n, not a newline.
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


def run_cargo():
    r = subprocess.run(["cargo", "test", "--manifest-path", "conformance/Cargo.toml", "--lib",
                        "--test", "contract_golden", "--test", "contract_audited_status",
                        "--test", "shape_expr", "--test", "synth_request_decline",
                        "--test", "structure_key_golden"],
                       capture_output=True, text=True, timeout=560)
    out = r.stdout + r.stderr
    # A mutation must COMPILE and RUN, or the failed set is meaningless (Copilot #311): a compile
    # error emits no per-test results and parsing zero FAILED reads as "killed nothing" — a silent
    # wrong-population defect inside the instrument. Require the test-output signature; fail loud.
    if "test result:" not in out:
        raise RuntimeError(
            f"cargo produced NO test results (exit {r.returncode}) — compile error or abort "
            f"before tests ran. First lines:\n" + "\n".join(out.splitlines()[:25]))
    # Match by last `::` component: lib unit tests are module-qualified, integration tests are not.
    return set(m.split("::")[-1] for m in re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M))


def read_bytes(p):
    with open(p, "rb") as fh:
        return fh.read()


def write_bytes(p, b):
    with open(p, "wb") as fh:
        fh.write(b)


def main():
    matrix = {}
    defect = 0
    for name, f, old, new in MUT:
        src = read_bytes(f)
        nl = b"\r\n" if b"\r\n" in src else b"\n"
        old = old.replace(b"\n", nl); new = new.replace(b"\n", nl)  # no-op for these single-line seeds
        assert src.count(old) == 1, f"NOT UNIQUE/absent: {name} {old!r} count={src.count(old)}"
        try:
            write_bytes(f, src.replace(old, new, 1))
            assert read_bytes(f).count(new) >= 1, f"SEED NOT APPLIED: {name}"  # convention 9
            failed = run_cargo()
        finally:
            write_bytes(f, src)
            assert read_bytes(f) == src, f"NOT RESTORED byte-exact: {name}"
        batch_hit = sorted(t for t in failed if t in BATCH)
        matrix[name] = batch_hit
        reached = name in failed
        if not reached:
            defect += 1
        spill = sorted(t for t in failed if t not in BATCH)
        print(f"[{BATCH[name]:<16}] seed->{name}: batch_killed={batch_hit}  intended_reached={reached}")
        print(f"                    spill(non-batch)={spill}")
    print("\n=== ISOLATION MATRIX (each seed should kill EXACTLY ONE batch test = its own) ===")
    ok = True
    for name, _f, _o, _n in MUT:
        hit = matrix[name]
        if hit != [name]:
            ok = False
        print(f"  {name:<46} kills {hit}  {'OK' if hit == [name] else '!! NOT ISOLATED'}")
    print(f"\nISOLATION: {'ALL EXACTLY-ONE' if ok else 'VIOLATIONS ABOVE'}")
    print(f"DEFECT RATE (intended test not reached on first attempt): {defect}/{len(MUT)}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
