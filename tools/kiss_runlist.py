"""Every clause-backing test must appear in a compiled test binary on some CI leg.

THE PROPERTY. `kiss_trace` answers *"does the spec name a test that exists in the
source?"* This answers the next question, which nothing asked until an audit found
six lint self-tests wired to no CI path and one of them red on `main`: **does
anything actually build and run it?** Naming a test is not running one.

WHY THIS IS A SEPARATE TOOL. `kiss_trace` is a static reader — it parses markdown
and `.rs` source and never invokes cargo. Whether a test ends up in a compiled test
binary is decided by `cfg` gates, feature flags and the target platform, i.e. by a
*build*, not by source text. A `#![cfg(feature = "cuda")]` test is not merely
un-run: on a leg without the feature it does not exist, so nothing static can
distinguish it from a test that runs fine. This tool therefore consumes a build
artifact — `cargo test -- --list`, captured per leg — and joins it to the clause
matrix. Same class boundary as the run-artifact joiner (KISS-CONFORM-6.1-0009b):
when the enforcing instrument cannot observe the property, the fix is a second
instrument of the right class, not a weaker claim.

WHAT IT REPORTS, and the distinction is the whole point:

    both legs      the backing is unconditional
    one leg only   the backing exists on exactly one platform — real coverage,
                   but coverage with a single point of failure. `#![cfg(windows)]`
                   files are here, including the §6.13-0006 freeze-gate slice.
    NO leg         the clause is backed by code nothing compiles. This FAILS.

Usage:
    python tools/kiss_runlist.py --leg ubuntu=ubuntu.txt --leg windows=windows.txt

Each file is the raw stdout of `cargo test -- --list` on that leg.
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt


def parse_list_file(path):
    """Bare test-fn names from one `cargo test -- --list` capture.

    Lines look like `harness::corpus::tests::corpus_is_deterministic: test` and
    `src\\lib.rs - runtime_gate (line 63): test` (doc-tests). The matrix names
    tests by bare fn name, so the module path is dropped — an ad-hoc check that
    compared bare names against these fully-qualified lines is exactly how an
    earlier pass of this audit nearly reported a false absence.
    """
    names = set()
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line.endswith(": test") and not line.endswith(": bench"):
                continue
            head = line.rsplit(":", 1)[0].strip()
            if " - " in head:            # a doc-test, not a test fn
                continue
            names.add(head.split("::")[-1])
    return names


def backing_tests(spec_dir, conf_dir):
    """{test_name: sorted(clause_ids)} for every test that backs a clause.

    Derived from `kiss_trace`, never restated: forward (the clause names the test)
    plus reverse (the test cites the clause), which is the same bidirectional rule
    §6.1 defines and `kiss_trace` implements.
    """
    results = []
    for stem in kt.SPECS:
        res = kt.DocResult(stem)
        path = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(path):
            raise SystemExit(f"missing spec file: {path}")
        kt.parse(path, res)
        results.append(res)
    harness = kt.discover_tests(conf_dir)
    out = {}
    for res in results:
        for cid, t, _ in res.matrix:
            if t in harness:
                out.setdefault(t, set()).add(cid)
    for t, info in harness.items():
        for cid in info["clauses"]:
            out.setdefault(t, set()).add(cid)
    return {t: sorted(c) for t, c in out.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--leg", action="append", default=[], metavar="NAME=PATH",
                    help="a `cargo test -- --list` capture, tagged with its CI leg")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    args = ap.parse_args()

    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")

    # A run with no legs must FAIL, not report a clean zero. If an artifact
    # download silently produced nothing, "0 tests missing" is the exact shape of
    # success-indistinguishable-from-failure this check exists to detect.
    if not args.leg:
        print("FAIL: no --leg given. A run with no build artifact cannot report "
              "coverage; it can only report that it has nothing to report.")
        return 2

    legs = {}
    for spec in args.leg:
        if "=" not in spec:
            print(f"FAIL: --leg expects NAME=PATH, got {spec!r}")
            return 2
        name, _, path = spec.partition("=")
        if not os.path.exists(path):
            print(f"FAIL: leg {name!r} capture not found: {path}")
            return 2
        legs[name] = parse_list_file(path)
        if not legs[name]:
            print(f"FAIL: leg {name!r} listed ZERO tests ({path}). An empty capture "
                  f"is a broken build step, not a suite with no tests.")
            return 2

    backing = backing_tests(spec_dir, conf_dir)

    print("=" * 68)
    print("  DOES ANYTHING RUN IT — clause-backing tests vs compiled test binaries")
    print("=" * 68)
    for name, tests in sorted(legs.items()):
        print(f"      leg {name:<10} {len(tests)} tests in its compiled binaries")
    print(f"      {len(backing)} tests back at least one clause")

    everywhere, single, nowhere = [], [], []
    for t in sorted(backing):
        on = sorted(n for n, tests in legs.items() if t in tests)
        if not on:
            nowhere.append((t, backing[t]))
        elif len(on) == len(legs):
            everywhere.append(t)
        else:
            single.append((t, on, backing[t]))

    print("-" * 68)
    print(f"  {len(everywhere):>4}  on EVERY leg — unconditional backing")
    print(f"  {len(single):>4}  on SOME legs — real coverage, single point of failure")
    for t, on, cids in single:
        print(f"          {t}  [{', '.join(on)}]  <- {', '.join(cids)}")
    print(f"  {len(nowhere):>4}  on NO leg — backed by code nothing compiles")
    for t, cids in nowhere:
        print(f"          {t}  <- {', '.join(cids)}")

    print("-" * 68)
    if nowhere:
        print(f"  RESULT: FAIL — {len(nowhere)} clause-backing test(s) are compiled by "
              f"no CI leg.\n          A clause backed by code nothing builds reads as "
              f"backed and is not.")
        # PROVENANCE, because this failure has a benign cause that looks identical.
        # A capture taken from a DIFFERENT commit than the spec/harness being checked
        # reports every test added since as absent. In CI both come from one
        # checkout, so this cannot happen there; run locally it happens easily, and
        # it produced a clean-looking list of 11 false absences while this tool was
        # being written.
        print("          CHECK THE CAPTURE'S PROVENANCE FIRST: a --list taken from a "
              "different\n          commit than this working tree reports every "
              "newer test as absent.")
        return 1
    print("  RESULT: CLEAN — every clause-backing test is compiled on at least one leg.")
    if single:
        print(f"  NOTE:   {len(single)} depend on a single leg. That is not a failure, but "
              f"it is\n          the whole evidence base for those clauses — if the leg is "
              f"skipped, so is the proof.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
