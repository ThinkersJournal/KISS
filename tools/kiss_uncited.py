#!/usr/bin/env python3
"""KISS-Conform uncited-test sweep — what is the harness already proving?

`kiss_trace.py` reports N executable tests that cite no clause: "real work the
matrix cannot see". They exist and they pass, so this is the cheapest genuine
coverage available — and, for exactly that reason, the easiest place to inflate
coverage dishonestly. Per #187 a MENTION counts as a citation, so bulk-adding
clause IDs would move ENFORCED without improving anything, and the coverage
ratchet would report it as progress.

So this tool produces CANDIDATES, never citations. It sorts the uncited
population by the strength of the evidence that a test asserts some clause's
actual obligation — the standard ruled on #191: **reading a clause's text as
data does not back it; asserting its requirement does.**

  declared   the test already names a clause in `§<sec>-<nnnn>` SHORT form, in
             its body or doc comment. The short form is deliberate — it does not
             match the citation grammar — so an author who wrote it was saying
             "this enforces that clause" without claiming the credit. This is
             the strongest available signal and still NOT a citation, because
             the short form omits the SUB-STANDARD: `§6.11-0002` resolves by the
             file's context, and resolving it is a judgement per row.

  plumbing   the test exercises the harness's own machinery — the JSON parser,
             hex helpers, the FFI loader and marshalling, corpus generation, the
             differ's bookkeeping. Real work, no clause obligation: nothing in
             the spec requires a hex round-trip.

  unclear    everything else. Needs a human. This bucket being large is the
             honest outcome, not a failure of the sweep.

LOCATION DOES NOT PREDICT THE BUCKET, and the first version of this sweep
assumed it did. `conformance/src/**` looks like helper code, but
`corpus.rs::rejects_reference_observed_provenance` asserts §6.5-0003's
circular-vector rule verbatim, and `structural.rs` carries NaN-propagation and
monoid-identity semantics straight out of KISS-Ops §6.11. Thirteen of the
thirty-seven src-module tests carry a declared clause.

Exit status is always 0: this reports, it does not gate.
"""
import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import kiss_trace as kt  # noqa: E402

# `§6.11-0002` / `§6.5-0004b` — the short form the citation grammar ignores.
RE_SHORT = re.compile(r"§\s*([0-9]+(?:\.[0-9]+)?)-([0-9]{4}[a-z]?)")

# Files whose tests exercise the harness's own machinery rather than a spec
# obligation. Kept as an explicit list rather than a path heuristic, because a
# path heuristic is what got this wrong the first time.
PLUMBING_FILES = {
    "conformance/src/json.rs",
    "conformance/src/harness/abi.rs",
    "conformance/src/harness/loader.rs",
    "conformance/src/harness/msvc.rs",
    "conformance/src/harness/differ.rs",
}


def sweep(spec_dir, conf_dir):
    results = []
    for stem in kt.SPECS:
        path = os.path.join(spec_dir, stem + ".md")
        res = kt.DocResult(stem)
        if os.path.exists(path):
            kt.parse(path, res)
        results.append(res)
    clause_test = {}
    for res in results:
        for cid, t, _ in res.matrix:
            clause_test[cid] = t
    harness = kt.discover_tests(conf_dir)
    named = set(clause_test.values())

    scopes = {}
    for root, dirs, files in os.walk(conf_dir):
        dirs[:] = [d for d in dirs if d != "target"]
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            try:
                src = open(os.path.join(root, fn), encoding="utf-8").read()
            except OSError:
                continue
            for m in kt.RE_RUST_TEST.finditer(src):
                brace = src.find("{", m.end() - 1)
                body = src[m.start():kt._body_span(src, brace)] if brace != -1 else m.group(0)
                scopes[m.group(1)] = body + "\n" + kt._leading_comment(src, m.start())

    rows = []
    for t in sorted(harness):
        info = harness[t]
        if info["clauses"] or t in named:
            continue  # cited, or backed forward by name — not this sweep's subject
        refs = sorted({f"§{a}-{b}" for a, b in RE_SHORT.findall(scopes.get(t, ""))})
        if refs:
            bucket = "declared"
        elif info["file"] in PLUMBING_FILES:
            bucket = "plumbing"
        else:
            bucket = "unclear"
        rows.append({"test": t, "file": info["file"], "bucket": bucket, "refs": refs})
    return rows


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    ap.add_argument("--bucket", default=None,
                    help="list only this bucket: declared | plumbing | unclear")
    a = ap.parse_args()
    root = os.path.dirname(HERE)
    rows = sweep(a.spec_dir or os.path.join(root, "spec"),
                 a.conformance_dir or os.path.join(root, "conformance"))

    counts = {b: sum(1 for r in rows if r["bucket"] == b)
              for b in ("declared", "plumbing", "unclear")}
    print("KISS-Conform uncited-test sweep  -  work the matrix cannot see")
    print("=" * 68)
    print(f"  {len(rows):4d} executable tests cite no clause and are not named by one")
    print("-" * 68)
    print(f"  {counts['declared']:4d} DECLARED  already name a clause in `§` short form  <- candidates")
    print(f"  {counts['plumbing']:4d} PLUMBING  harness machinery, no clause obligation")
    print(f"  {counts['unclear']:4d} UNCLEAR   needs a human")
    print("-" * 68)
    print("  A candidate is NOT a citation. The short form omits the SUB-STANDARD,")
    print("  so resolving `§6.11-0002` to a full ID is a per-row judgement — and per")
    print("  #191, a test backs a clause only where it asserts that clause's own")
    print("  obligation, not where it merely mentions or reads it.")
    for b in ("declared", "plumbing", "unclear"):
        if a.bucket and a.bucket != b:
            continue
        if not a.bucket and b != "declared":
            continue
        print()
        print(f"  --- {b} ---")
        for r in rows:
            if r["bucket"] != b:
                continue
            short = r["file"].replace("conformance/", "").replace("tests/", "t/").replace("src/", "s/")
            print(f"    {short:34s} {r['test'][:44]:46s} {' '.join(r['refs'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
