#!/usr/bin/env python3
"""
kiss_vocab.py — the spec <-> reference-binding enumeration lint.

A document lint, complementary to tools/kiss_tables.py. Where kiss_tables checks
that a shared enumeration agrees with ITSELF across the spec (the dtype set, the
determinism enum), this checks that a CLOSED enumeration the spec pins agrees with
the way the reference conformance harness BINDS it — the op-family code set, and
(extensibly) the OpAttrs sub-vocabularies.

Why this matters, and why it is not the same lint as kiss_tables: some closed sets
are restated in several spec files (dtypes, the determinism enum) — kiss_tables
catches a re-fork there. Others are owned by ONE clause and only *referenced* by
tag elsewhere (the 24 op-family codes live only in KISS-Classify §6.5-0006; the
OpAttrs value enums live only in KISS-Ops §6.19 — Grammar/Contract embed OpAttrs
opaquely). For those, a cross-document consistency lint has nothing to disagree
with — it is teethless. The drift that CAN happen is between the pinned set and
the reference implementation that binds it: change the spec's op-family list
without the codec (or vice versa) and the reference reader silently accepts or
rejects the wrong token. This lint is the tooth for exactly that.

It is a reference-BINDING consistency check, not a conformance test: it does not
prove the harness computes a right answer, only that its bound vocabulary matches
the spec it claims to implement. A drift in either direction fails the lint.

Exit status is 0 on agreement, 1 on drift, so it is a CI gate. Stdlib only.

Usage:
  python tools/kiss_vocab.py                  # check
  python tools/kiss_vocab.py --emit-coverage  # print the clauses it enforces
"""
from __future__ import annotations

import argparse
import os
import re
import sys

# A 3-letter lowercase op-family code (KISS-Classify §6.5-0006).
FAMILY_CODE = re.compile(r"`([a-z]{3})`")

NUMBER_WORDS = {
    "twenty-two": 22, "twenty-three": 23, "twenty-four": 24, "twenty-five": 25,
}

# Each entry: a closed set the spec pins and the harness binds, plus the clause a
# drift fails. (name, clause_id, spec-extractor, harness-file, harness-const).
COVERS = [("KISS-CLASSIFY-6.5-0006",
           "the 24 op-family codes drift between Classify §6.5-0006 and the "
           "reference structure_key codec (OP_FAMILIES)")]


def _markdown_tables(text):
    """Yield each contiguous markdown table as its raw text block."""
    lines, block = text.splitlines(), []
    for line in lines:
        if line.strip().startswith("|") and line.strip().endswith("|"):
            block.append(line)
        elif block:
            yield "\n".join(block)
            block = []
    if block:
        yield "\n".join(block)


def spec_op_families(classify_text):
    """The op-family code set pinned by KISS-Classify §6.5-0006, plus the count word.

    §6.5-0006 says the domain is "exactly the twenty-four categories of the table
    above, each spelled by its 3-letter token code". The table is a two-pair
    layout (`| desc | code | desc | code |`); the op-family table is the one that
    carries the most 3-letter backtick codes.
    """
    best = []
    for block in _markdown_tables(classify_text):
        codes = FAMILY_CODE.findall(block)
        if len(codes) > len(best):
            best = codes
    m = re.search(
        r"op-family-tag domain MUST be exactly the\s+([a-z-]+)\s+categories",
        classify_text)
    count_word = m.group(1) if m else None
    return best, count_word


def harness_op_families(rs_text):
    """The op-family codes bound by the reference codec: the string literals in
    `pub const OP_FAMILIES: [&str; N] = [ "gem", ... ];`."""
    m = re.search(r"OP_FAMILIES\s*:\s*\[&str;\s*\d+\]\s*=\s*\[(.*?)\]", rs_text, re.S)
    if not m:
        return None
    return re.findall(r'"([a-z0-9]+)"', m.group(1))


def check(spec_dir, conf_dir):
    violations = []
    classify = os.path.join(spec_dir, "classify.md")
    rs = os.path.join(conf_dir, "src", "structure_key.rs")
    if not os.path.exists(classify):
        return [f"missing {classify}"]
    if not os.path.exists(rs):
        return [f"missing {rs} (the reference codec that binds the op-family set)"]

    spec_codes, count_word = spec_op_families(open(classify, encoding="utf-8").read())
    harness_codes = harness_op_families(open(rs, encoding="utf-8").read())
    if harness_codes is None:
        return ["could not find OP_FAMILIES in structure_key.rs"]

    spec_set, harness_set = set(spec_codes), set(harness_codes)

    # the declared count word must match the actual pinned size
    if count_word in NUMBER_WORDS and NUMBER_WORDS[count_word] != len(spec_set):
        violations.append(
            f"§6.5-0006 says '{count_word}' ({NUMBER_WORDS[count_word]}) but the table "
            f"lists {len(spec_set)} distinct codes")
    if len(spec_codes) != len(spec_set):
        dup = sorted({c for c in spec_codes if spec_codes.count(c) > 1})
        violations.append(f"the §6.5 op-family table has duplicate code(s): {dup}")
    if len(harness_codes) != len(harness_set):
        dup = sorted({c for c in harness_codes if harness_codes.count(c) > 1})
        violations.append(f"OP_FAMILIES has duplicate code(s): {dup}")

    if spec_set != harness_set:
        only_spec = sorted(spec_set - harness_set)
        only_harness = sorted(harness_set - spec_set)
        parts = []
        if only_spec:
            parts.append(f"in §6.5-0006 but not the codec: {only_spec}")
        if only_harness:
            parts.append(f"in the codec but not §6.5-0006: {only_harness}")
        violations.append("op-family set drift — " + "; ".join(parts))

    return violations, sorted(spec_set)


def main():
    ap = argparse.ArgumentParser(description="KISS spec<->reference-binding enum lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note)")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    result = check(spec_dir, conf_dir)
    if isinstance(result, list):
        print("KISS vocab lint — FATAL")
        for v in result:
            print(f"  - {v}")
        return 1
    violations, codes = result

    print("KISS spec<->binding enumeration lint")
    print("=" * 68)
    print(f"  op-family codes (KISS-CLASSIFY-6.5-0006): {len(codes)}")
    print(f"    {' '.join(codes)}")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} disagreement(s) spec <-> reference codec:")
        for v in violations:
            print(f"      - {v}")
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  the spec's op-family set and the reference codec agree.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
