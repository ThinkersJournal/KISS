"""#278 -- the generic re-runnable isolation-matrix driver CORE (proven_matrix.py).

The batch wrappers (proven_batch1_matrix.py, proven_batch2_matrix.py) supply (BATCH, MUT); this core
does the running, and it carries the two hardenings the #326 review found MISSING from the hand-copied
batch-1 driver -- so both wrappers get them BY CONSTRUCTION, which is what retires the harmonize marker
that used to sit at the top of batch 1's driver:

  * DERIVED targets, not hardcoded. `derive_targets` reads the cargo `--test <bin>` / `--lib` set from
    kiss_trace.discover_tests(). A hardcoded target list is a silently-narrowed population -- the same
    defect as `--no-fail-fast` one layer up, and a wrapper a narrowed list could pass by READING is
    exactly what a diffed-matrix equivalence check exists to catch.
  * BASELINE GATE. The unmutated target set must be all-green before any seed (else exit 2, nothing
    seeded). A pre-existing red rides every mutation's kill set; if it is a batch test, "kills exactly
    one" is satisfied by the stale red, not the mutation -- a false PROVEN. An operator instruction to
    "run on a clean tree" is not a check; this is.
  * `--no-fail-fast`. Without it cargo stops after the first failing TARGET, so a mutation whose victim
    is in an early target truncates the run and later targets never execute -- "kills exactly one" then
    ranges over a truncated, mutation-dependent population (being in the run SET is not being in the
    run). With it, every target runs every time and the kill set is COMPLETE.

For each mutation the core seeds a one-site edit of the clause's IMPLEMENTATION in conformance/src/,
runs the targets UNFILTERED, records the kill set, and restores byte-exact (seed-applied + restore-exact
asserted, convention 9). It prints per-mutation kills + spill, the isolation matrix (each seed must kill
EXACTLY ONE batch test = its own), and the demonstration defect rate.

Import-safe: this module does nothing at import. run() is called only from a wrapper's __main__.
Exit codes: 0 isolation all-exactly-one; 1 isolation violation; 2 baseline not green (nothing mutated).
"""
import subprocess, re, sys, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace


def derive_targets(batch, conf="conformance"):
    """The cargo targets holding the batch's proving tests, DERIVED from discover_tests -- never
    hardcoded. Integration tests (conformance/tests/X.rs) -> `--test X`; lib unit tests
    (#[cfg(test)] in conformance/src/Y.rs) -> `--lib`. Raises if a proving test cannot be located,
    because a silently-dropped target is the population narrowing this whole split exists to remove."""
    harness = kiss_trace.discover_tests(conf)
    missing = sorted(t for t in batch if t not in harness)
    if missing:
        raise RuntimeError(f"proving tests not found by discover_tests (cannot derive targets): {missing}")
    bins = set()
    need_lib = False
    for t in batch:
        parts = harness[t]["file"].replace("\\", "/").split("/")
        if "tests" in parts:
            bins.add(os.path.splitext(parts[-1])[0])
        elif "src" in parts:
            need_lib = True
        else:
            raise RuntimeError(f"cannot derive a cargo target for {t} in {harness[t]['file']}")
    return (["--lib"] if need_lib else []) + [a for b in sorted(bins) for a in ("--test", b)]


def run_cargo(targets):
    r = subprocess.run(["cargo", "test", "--no-fail-fast",
                        "--manifest-path", "conformance/Cargo.toml"] + targets,
                       capture_output=True, text=True, timeout=560)
    out = r.stdout + r.stderr
    # A mutation must COMPILE and RUN, or the failed set is meaningless (Copilot #311): a compile error
    # emits no per-test results and parsing zero FAILED reads as "killed nothing". Require the signature.
    if "test result:" not in out:
        raise RuntimeError(
            f"cargo produced NO test results (exit {r.returncode}) -- a compile error or an abort "
            f"before tests ran; the failed set would be a false zero. First lines:\n"
            + "\n".join(out.splitlines()[:25]))
    # Match by last `::` component: lib unit tests are module-qualified, integration tests are not.
    return set(m.split("::")[-1] for m in re.findall(r"^test (\S+) \.\.\. FAILED", out, re.M))


def _read(p):
    with open(p, "rb") as fh:
        return fh.read()


def _write(p, b):
    with open(p, "wb") as fh:
        fh.write(b)


def run(batch, mut):
    targets = derive_targets(batch)
    print(f"DERIVED targets (from discover_tests, not hardcoded): {' '.join(targets)}")
    base = run_cargo(targets)  # BASELINE GATE
    if base:
        print(f"BASELINE NOT GREEN -- {len(base)} pre-existing failure(s): {sorted(base)}")
        print("Refusing to seed: every mutation's kill set would inherit these. Clean the tree first.")
        return 2
    print("BASELINE: unmutated target set all-green (0 failures) -- each kill below is the mutation's own.\n")
    matrix = {}
    defect = 0
    for name, f, old, new in mut:
        src = _read(f)
        nl = b"\r\n" if b"\r\n" in src else b"\n"
        old = old.replace(b"\n", nl); new = new.replace(b"\n", nl)  # no-op for single-line seeds
        assert src.count(old) == 1, f"NOT UNIQUE/absent: {name} {old!r} count={src.count(old)}"
        try:
            _write(f, src.replace(old, new, 1))
            assert _read(f).count(new) >= 1, f"SEED NOT APPLIED: {name}"  # convention 9
            failed = run_cargo(targets)
        finally:
            _write(f, src)
            assert _read(f) == src, f"NOT RESTORED byte-exact: {name}"
        batch_hit = sorted(t for t in failed if t in batch)
        matrix[name] = batch_hit
        reached = name in failed
        if not reached:
            defect += 1
        spill = sorted(t for t in failed if t not in batch)
        print(f"[{batch[name]:<18}] seed->{name}: batch_killed={batch_hit}  intended_reached={reached}")
        print(f"                    spill(non-batch)={spill}")
    print("\n=== ISOLATION MATRIX (each seed should kill EXACTLY ONE batch test = its own) ===")
    ok = True
    for name, _f, _o, _n in mut:
        hit = matrix[name]
        if hit != [name]:
            ok = False
        print(f"  {name:<46} kills {hit}  {'OK' if hit == [name] else '!! NOT ISOLATED'}")
    print(f"\nISOLATION: {'ALL EXACTLY-ONE' if ok else 'VIOLATIONS ABOVE'}")
    print(f"DEFECT RATE (intended test not reached on first attempt): {defect}/{len(mut)}")
    return 0 if ok else 1
