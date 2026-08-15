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

  declared   an author names a clause in `§<sec>-<nnnn>` SHORT form anywhere from
             the END OF THE PREVIOUS TEST up to and including this one — its body,
             its doc comment, or a header comment sitting in that gap. A header
             therefore covers the NEXT test only; it does not carry forward to the
             rest of a group. The short form is deliberate: it does not match the
             citation grammar, so the author said "this enforces that clause"
             without claiming the credit. Strongest available signal, and still
             NOT a citation — the short form omits the SUB-STANDARD, so
             `§6.11-0002` resolves only by the file's context, per row, by a
             reader.

             THE SCOPE IS WIDER THAN `kiss_trace`'S, ON PURPOSE. The gate reads
             the CONTIGUOUS `//` run above `#[test]`, because that is what grants
             credit, and this sweep must never widen THAT. But a header separated
             by a blank line is invisible to a contiguous run, and 27 of the 85
             rows first bucketed "needs a human" carried exactly that — the first
             cut understated DECLARED by a third.

             AND IT IS DELIBERATELY NOT WIDER STILL. Carrying a header forward
             until superseded was considered and MEASURED: it moves 53 further
             rows into `declared`, and the additions are plainly wrong — the hash
             helper `fnv1a64_is_deterministic_and_input_sensitive` inherits
             §6.4-0010 from its MODULE doc comment, and `exact_byte_comparator`
             inherits §6.0-0001 from `lib.rs`'s header. Proximity to a file's
             prose is not declaration, and inflating the strongest bucket with
             false positives is the failure this tool exists to avoid.

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

# THE AUTHOR SOMETIMES ANSWERS THE QUESTION THIS TOOL ASKS. The short form's whole
# problem is that it omits the sub-standard, so `§6.8-0004` has to be resolved by a
# reader — but authors frequently write the sub-standard immediately before it:
#
#     // magnitude (Conform §6.8-0004: the declared, derived tolerance — not a byte
#
# `RE_SHORT` starts at the `§` and discards the word in front of it, so the
# STRONGEST available evidence — the author's own disambiguation — was thrown away
# and inference guessed in its place. Measured: 7 declared rows carry a qualifier,
# and 5 resolve to a DIFFERENT sub-standard than the file's home doc implies.
#
# This is not a tie-breaker, it is a different KIND of evidence. `§6.8-0004` in an
# ops-homed file infers to `KISS-OPS-6.8-0004` — `erf`/`lgamma` special-function
# atoms — in three tests about floating-point atomic ordering. The author wrote
# `Conform`, and `KISS-CONFORM-6.8-0004` is the comparator clause those tests
# exercise. Inference produced a clause on the wrong SUBJECT, and nothing in the
# row would have shown it.
SUB_WORDS = ("classify", "ops", "conform", "contract", "grammar",
             "emit", "consume", "announce", "synth", "dispatch")
RE_QUALIFIED = re.compile(
    r"\b(" + "|".join(SUB_WORDS) + r")\b[\s\-]*§\s*([0-9]+(?:\.[0-9]+)?)-([0-9]{4}[a-z]?)",
    re.I,
)

# Wording that may mean the test does NOT enforce the clause it names. This FLAGS
# for a reader; it does not rebucket. The measurement is the reason: matching
# `kiss_cites`' RE_CONTRASTIVE against the ref's own line hits 14 declared rows and
# only TWO are real disclaimers —
#
#     "(caught by the differential + SYNTH §6.5-0004b, not this lint)"
#     "Out-of-contract degenerate (§6.11-0004 does not pin)"
#
# The other twelve are the negation attaching to the CLAUSE'S CONTENT rather than
# to the test's relationship with it: `max/min monoids MUST be NaN-propagating (not
# IEEE maxNum)` is the obligation being asserted, not a disclaimer of it. Same
# words, opposite meaning, and the difference is what the `not` attaches to — which
# a regex cannot see. Rebucketing on it would silently drop twelve legitimate
# candidates while looking like a precision improvement, so the narrow phrases
# below flag only the shapes that speak about THIS test's scope.
RE_WORDING = re.compile(
    r"(not this\b|does not pin\b|not enforced here\b|not (?:this )?lint\b)",
    re.I,
)

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

    scopes, decl_scopes = {}, {}
    for root, dirs, files in os.walk(conf_dir):
        dirs[:] = [d for d in dirs if d != "target"]
        for fn in sorted(files):
            if not fn.endswith(".rs"):
                continue
            try:
                src = open(os.path.join(root, fn), encoding="utf-8").read()
            except OSError:
                continue
            # TWO SCOPES, deliberately different, because they answer different
            # questions. `kiss_trace`'s CITATION scope is the body plus the
            # CONTIGUOUS `//` run directly above the `#[test]` — that is what
            # credits a clause, and this sweep must not widen it, or it would
            # report coverage the gate does not grant.
            #
            # The DECLARATION scope is wider: everything since the END of the
            # previous test. An author who writes `// ---- declines (§6.10-0006 …)`
            # above a test has declared the clause for THAT test, and a blank line
            # makes it invisible to the contiguous run. Measured: 27 of the 85 rows
            # first bucketed "needs a human" carry exactly that, so the first cut
            # understated DECLARED by a third and overstated UNCLEAR by the same.
            #
            # NOT carried forward past the next test. Doing so moves 53 further
            # rows in, and they are false: the hash helper `fnv1a64_is_…` inherits
            # §6.4-0010 from its MODULE doc comment. A header covers what follows
            # it up to the next test, never a whole file.
            #
            # THE FIRST TEST IN A FILE IS THE EXCEPTION, and getting it wrong is how
            # this tool broke its own stated rule. "Everything since the end of the
            # previous test" has no previous test to start from, so `prev_end = 0`
            # made the scope THE ENTIRE FILE ABOVE — every clause named anywhere in
            # the implementation body, the module doc comment, the imports. That is
            # exactly the whole-file inheritance the paragraph above disclaims, and
            # it named the very example it produced: `fnv1a64_empty_is_the_offset_basis`
            # is the first test in `expressibility.rs` and inherited EIGHT refs from
            # the file above it.
            #
            # With no previous test to bound a header's reach, there is no evidence
            # about how far one reaches, so fall back to the NARROW contiguous run —
            # the same scope `kiss_trace` credits. Understating a first test's
            # declarations is a row that needs a human; overstating them is a false
            # citation candidate that reads as authored.
            #
            # Measured at origin/main efe111c: refs 506 -> 330, declared bucket
            # 59 -> 50. Thirteen rows are first-in-file and contributed 45% of all
            # declared refs before this. `empty_reduction_is_monoid_identity` alone
            # went 14 -> 1, and the 1 is the clause its own comment names.
            prev_end = 0
            for m in kt.RE_RUST_TEST.finditer(src):
                brace = src.find("{", m.end() - 1)
                end = kt._body_span(src, brace) if brace != -1 else m.end()
                body = src[m.start():end] if brace != -1 else m.group(0)
                lead = kt._leading_comment(src, m.start())
                scopes[m.group(1)] = body + "\n" + lead
                gap = lead if prev_end == 0 else src[prev_end:m.start()]
                decl_scopes[m.group(1)] = gap + "\n" + body
                prev_end = end

    rows = []
    for t in sorted(harness):
        info = harness[t]
        if info["clauses"] or t in named:
            continue  # cited, or backed forward by name — not this sweep's subject
        scope = decl_scopes.get(t, "")
        refs = sorted({f"§{a}-{b}" for a, b in RE_SHORT.findall(scope)})
        # The author's own disambiguation, where they gave one. Keyed by short ref
        # so a row can carry both qualified and bare refs; the caller resolving a
        # bare one still needs the home doc, but must never override a qualified.
        qualified = {f"§{a}-{b}": s.lower()
                     for s, a, b in RE_QUALIFIED.findall(scope)}
        # Lines where this test says something about its OWN relationship to a
        # clause it names. A flag for a reader, never a rebucket.
        wording = sorted({ln.strip()[:100] for ln in scope.splitlines()
                          if RE_SHORT.search(ln) and RE_WORDING.search(ln)})
        if refs:
            bucket = "declared"
        elif info["file"] in PLUMBING_FILES:
            bucket = "plumbing"
        else:
            bucket = "unclear"
        rows.append({"test": t, "file": info["file"], "bucket": bucket, "refs": refs,
                     "qualified": qualified, "wording": wording})
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
    nqual = sum(1 for r in rows if r.get("qualified"))
    nword = sum(1 for r in rows if r.get("wording"))
    print(f"  {nqual:4d} rows carry the author's OWN sub-standard word (`Conform §6.8-0004`)")
    print(f"  {nword:4d} rows say something about their own scope near a ref  <- read before citing")
    print("-" * 68)
    print("  A candidate is NOT a citation. The short form omits the SUB-STANDARD,")
    print("  so resolving `§6.11-0002` to a full ID is a per-row judgement — and per")
    print("  #191, a test backs a clause only where it asserts that clause's own")
    print("  obligation, not where it merely mentions or reads it.")
    print("  A ref shown as `conform §6.8-0004` was QUALIFIED BY THE AUTHOR. That is")
    print("  evidence, not inference — do not override it with the file's home doc.")
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
            shown = " ".join(f"{r['qualified'][x]} {x}" if x in r.get("qualified", {}) else x
                             for x in r["refs"])
            print(f"    {short:34s} {r['test'][:44]:46s} {shown}")
            for w in r.get("wording", []):
                print(f"    {'':34s} {'':46s} ^ scope note: {w}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
