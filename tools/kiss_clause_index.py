#!/usr/bin/env python3
"""Emit KISS's clause set as a machine-readable index, for downstream projects to vendor.

WHY THIS EXISTS
---------------
Fuel cites 16 distinct KISS clause ids and no gate on either side can tell whether those
clauses still exist. A citation to a clause KISS has since RENUMBERED sits indefinitely:
true when written, with no detector anywhere. `KISS-CONTRACT-6.1-0008` is the live example
of an id whose meaning moved this week.

A downstream project that vendors this index at a pinned ref turns "does this clause still
exist?" into a DIFF, and a renumbering into a RED.

WHAT IT IS NOT
--------------
This is an EXPORT, not a gate. It is deliberately a separate tool from `kiss_trace.py`:
that one BLOCKS a build, and an export's failure mode must not be able to reach it.

It reuses `kiss_trace`'s parsers rather than re-implementing them. A second regex for the
same construct is a second thing to drift.

⚠️  THE SELF-CHECK IS A SET COMPARISON, NOT A COUNT
---------------------------------------------------
Every clause has exactly one §9 matrix row (measured 936:936). That invariant is what makes
the index complete BY CONSTRUCTION rather than by hope, so it is asserted before emitting.

It is asserted as a SET comparison. A count check would pass whenever two DIFFERENT sets
project to the same number -- a clause defined but unmatrixed, plus a matrix row for a
clause that no longer exists, cancel exactly and leave the totals equal. Counting is a
projection; only the set check sees a swap.

THREE OUTCOMES, kept distinct so "could not measure" never reads as "measured clean":
    0  OK        index emitted
    1  MISMATCH  the invariant is broken -- defs and matrix rows disagree. A real defect.
    2  REFUSED   the input could not be read at all. NOT a finding about the corpus.
"""
import sys, os, pathlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import kiss_trace as kt

SPEC_SUBDIR = "spec"


def build(root):
    """Return (defs, matrix, stems). `kiss_trace.SPECS` is the canonical clause-bearing set:
    `spec/namespaces/*` and `spec/umbrella.md` define no clauses and are correctly outside it."""
    # RAISES on a missing spec file rather than returning a sentinel: a sentinel gave the
    # caller a second REFUSED route, and a test covering one route made the other look
    # covered. One way to fail is one way to test.
    #
    # DUPLICATES ARE REFUSED, NOT DEDUPLICATED. `defs[cid] = ...` keeps the LAST copy and
    # drops the first silently, which would defeat the very invariant this tool exports --
    # "exactly one definition and one matrix row per clause". Worse, the SET check below
    # cannot see it: two definitions of one id collapse to one key, so `defs` and `matrix`
    # still match and the index looks complete. The asymmetry check is blind to duplication
    # by construction, so duplication needs its own detector.
    defs, matrix, stems = {}, {}, []
    dupes = []
    for stem in kt.SPECS:
        path = os.path.join(str(root), SPEC_SUBDIR, stem + ".md")
        if not os.path.exists(path):
            raise FileNotFoundError(path)
        res = kt.DocResult(stem)
        kt.parse(path, res)
        stems.append(stem)
        for cid, ln, tests in res.body:
            if cid in defs:
                dupes.append(f"{cid}: defined twice (lines {defs[cid][0]} and {ln})")
            defs[cid] = (ln, tests)
        for cid, test, _ln in res.matrix:
            if cid in matrix:
                dupes.append(f"{cid}: two matrix rows (`{matrix[cid]}` and `{test}`)")
            matrix[cid] = test
    if dupes:
        raise ValueError("duplicate clause ids in the corpus: " + "; ".join(sorted(dupes)))
    return defs, matrix, stems


def main(argv):
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path(__file__).resolve().parents[1]
    try:
        defs, matrix, files = build(root)
    except Exception as e:                      # noqa: BLE001 -- REFUSED, not a finding
        print(f"REFUSED: could not parse the spec corpus: {e}", file=sys.stderr)
        return 2
    # POSITIVE CONTROL, run THROUGH the same parse the index is built from -- not beside it.
    # A control that runs by a different mechanism cannot fail the way the instrument fails.
    control = "KISS-OPS-6.19-0005"
    if control not in defs:
        print(f"REFUSED: positive control {control} not found -- the parse is not working, "
              f"so an empty or short index would be a lie about the corpus", file=sys.stderr)
        return 2

    # ⚠️  SET comparison, not a count. Print the counts AND reconcile them.
    only_def = sorted(set(defs) - set(matrix))
    only_mat = sorted(set(matrix) - set(defs))
    print(f"sub-standards: {len(files)}   clause definitions: {len(defs)}   "
          f"matrix rows: {len(matrix)}", file=sys.stderr)
    if only_def or only_mat:
        print("MISMATCH: the clause set and the §9 matrix disagree.", file=sys.stderr)
        for cid in only_def:
            print(f"  defined, NO matrix row: {cid}", file=sys.stderr)
        for cid in only_mat:
            print(f"  matrix row, NO definition: {cid}", file=sys.stderr)
        print(f"  ({len(only_def)} + {len(only_mat)} asymmetric; counts alone would have "
              f"read {len(defs)} vs {len(matrix)})", file=sys.stderr)
        return 1

    print("clause_id\tsub_standard\tsection\tnamed_test")
    for cid in sorted(defs):
        m = kt.RE_IDPART.match(cid)
        sub, section = (m.group(1), m.group(2)) if m else ("", "")
        print(f"{cid}\t{sub}\t{section}\t{matrix[cid]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
