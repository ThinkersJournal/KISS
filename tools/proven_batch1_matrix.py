"""#278 batch 1 — the re-runnable isolation-matrix driver behind conformance/PROVEN_BATCH1.md.

For each of the ten batch clauses it seeds a one-site mutation of the clause's IMPLEMENTATION
(the obligation subject, convention 15) in conformance/src/, runs the four relevant test
binaries UNFILTERED, records which tests fail, and restores the source BYTE-FOR-BYTE. It asserts
each seed applied (convention 9) and each restore is byte-exact, and prints:
  * per-mutation: which batch test(s) it killed + every test it killed (batch and non-batch);
  * the isolation matrix — each mutation must kill EXACTLY ONE batch test (its own);
  * the demonstration defect rate — how many intended tests failed to redden on the first run.

This driver MUTATES source while it runs, so it is import-safe (guarded below) and reverts on
every path via try/finally. Run from the batch worktree ROOT:  python tools/proven_batch1_matrix.py

WARNING: run only on a clean tree; an interrupt mid-cargo leaves the current seed applied (the
finally restores it, but kill -9 does not). `git diff conformance/src` must be empty after.
"""
import io, subprocess, re, sys

SK = "conformance/src/structure_key.rs"; IT = "conformance/src/integer.rs"
ST = "conformance/src/structural.rs"; CT = "conformance/src/contract.rs"

BATCH = {  # proving test -> clause (the ten demonstrated; 6.1-0008 deferred, see PROVEN_BATCH1.md)
    "reject_unknown_dtype": "CLASSIFY-6.1-0001", "reject_unknown_op_family": "CLASSIFY-6.5-0006",
    "reject_wrong_field_count": "CLASSIFY-6.7-0009", "test_classify_work_class_element_count": "CLASSIFY-6.5-0007",
    "test_ops_add_sub_mul_wrapping": "OPS-6.4-0001", "test_ops_neg_integer": "OPS-6.4-0003",
    "scan_is_length_preserving": "OPS-6.11-0003", "scatter_oob_writes_are_skipped": "OPS-6.11-0005",
    "scatter_atomic_max_min_nan_propagating": "OPS-6.11-0010", "test_contract_version_value_pinned": "CONTRACT-6.1-0008"}

# (proving_test, file, old, new) — each `old` occurs exactly once; the mutation attacks the
# clause's obligation. Byte strings so the seed/restore is line-ending exact.
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


def run_cargo():
    r = subprocess.run(["cargo", "test", "--manifest-path", "conformance/Cargo.toml",
                        "--test", "structure_key_golden", "--test", "integer_semantics",
                        "--test", "structural_access", "--test", "contract_framing"],
                       capture_output=True, text=True, timeout=560)
    out = r.stdout + r.stderr
    return set(re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M)), out


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
        # Match the file's line ending: patterns are written with \n, the worktree may be \r\n.
        nl = b"\r\n" if b"\r\n" in src else b"\n"
        old = old.replace(b"\n", nl)
        new = new.replace(b"\n", nl)
        assert src.count(old) == 1, f"NOT UNIQUE/absent: {name} {old!r} count={src.count(old)}"
        try:
            write_bytes(f, src.replace(old, new, 1))
            assert read_bytes(f).count(new) >= 1, f"SEED NOT APPLIED: {name}"  # convention 9
            failed, _ = run_cargo()
        finally:
            write_bytes(f, src)                                   # byte-exact restore, every path
            assert read_bytes(f) == src, f"NOT RESTORED byte-exact: {name}"
        batch_hit = sorted(t for t in failed if t in BATCH)
        matrix[name] = batch_hit
        reached = name in failed
        if not reached:
            defect += 1
        print(f"[{BATCH[name]:<18}] seed->{name}: batch_killed={batch_hit}  "
              f"(all_failed={sorted(failed)})  intended_reached={reached}")
    print("\n=== ISOLATION MATRIX (each seed should kill EXACTLY ONE batch test = its own) ===")
    ok = True
    for name, _f, _o, _n in MUT:
        hit = matrix[name]
        if hit != [name]:
            ok = False
        print(f"  {name:<40} kills {hit}  {'OK' if hit == [name] else '!! NOT ISOLATED'}")
    print(f"\nISOLATION: {'ALL EXACTLY-ONE' if ok else 'VIOLATIONS ABOVE'}")
    print(f"DEFECT RATE (intended test not reached on first attempt): {defect}/{len(MUT)}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
