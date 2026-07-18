#!/usr/bin/env python3
"""
kiss_tables.py — the KISS shared-enumeration consistency lint.

A *document* lint, complementary to tools/kiss_trace.py: where kiss_trace checks
that each clause maps to an executable test, this checks that the several places
the spec re-lists a shared, owned enumeration do not DRIFT out of agreement.

KISS deliberately re-states some sets in more than one place — an owner's
normative table, an informative overview table, a shared-vocabulary anchor
sentence — "so the Ops binary encoding is self-contained" (ops §6.16 preamble).
The cost is that an additive amendment can update one copy and miss another. That
is not hypothetical: PRs #30/#31 added `s16`/`u16`/`u64` to the Classify §6.1
dtype table and the Ops §6.16 layout table, but left the Ops §2.8 shared-anchor
list at seventeen tokens. A markdown check finds that class of drift the moment
it lands, at the PR, instead of years later by inspection.

THE DTYPE SET is the first (and, today, only) enumeration checked. Its owner and
single source of truth is KISS-CLASSIFY-6.1-0001, whose own words are "The scalar
dtype set MUST be **exactly** the twenty tokens in the table above (...)". This
lint extracts that authoritative set and asserts that:

  1. every FULL dtype markdown table in the suite carries exactly it (the
     informative §2.6 table, the normative §6.1 table, the Ops §6.16 layout
     table — detected structurally, by a first column that is ≥ 15 dtype tokens,
     so a subset table is never mistaken for a full one);
  2. every FULL dtype inline list carries exactly it — the §6.1-0001 parenthetical
     itself, and the Ops §2.8 "**dtype** tokens (...)" shared anchor — detected by
     their specific introducing phrases, so a subset list such as "dtypes
     (`f16`, `f32`, `f64`)" is never checked as if it were the whole set;
  3. the count word in §6.1-0001 ("twenty") equals the set's actual size, so a
     token added to the table without updating the prose count is caught too.

Exit status is 0 when every site agrees and 1 on any drift, so the checker is a
CI gate. Stdlib only.

Usage:  python tools/kiss_tables.py [--spec-dir spec]
"""
from __future__ import annotations

import argparse
import os
import re
import sys

NUMBER_WORDS = {
    "seventeen": 17, "eighteen": 18, "nineteen": 19, "twenty": 20,
    "twenty-one": 21, "twenty-two": 22, "twenty-three": 23,
}

# The umbrella + the nine sub-standards (umbrella defines no clauses, but the
# shared enums it narrates may appear in any of the ten files).
SPECS = ["umbrella", "announce", "classify", "ops", "grammar", "contract",
         "synth", "consume", "emit", "conform"]


def _is_dtype_token(w):
    """A dtype token is a lowercase alnum word that contains a digit (f16, bf16,
    s16, e4m3, c64, b1, ...) or is exactly `bool`. This admits the interleaved
    FP8/FP4 names (e4m3, e5m2, e2m1) and excludes digit-free field names (`rank`,
    `structure_key`) and prose, so a mixed backtick list yields only the dtypes.
    """
    return bool(re.fullmatch(r"[a-z][a-z0-9]*", w)) and (
        any(c.isdigit() for c in w) or w == "bool")


def backtick_tokens(text):
    """Every dtype token inside a backtick span in `text`, in order.

    Handles both spellings the spec uses: comma-separated individual spans
    (`f16`, `bf16`, ...) and one space-separated span (`f16 bf16 ...`).
    """
    out = []
    for span in re.findall(r"`([^`]+)`", text):
        out.extend(w for w in re.split(r"[\s,]+", span.strip()) if _is_dtype_token(w))
    return out


def authoritative_set(classify_text):
    """The dtype set owned by KISS-CLASSIFY-6.1-0001, plus its declared count word.

    Returns (tokens_in_order, count_word_or_None). The clause text is:
      "The scalar dtype set MUST be **exactly** the twenty tokens in the table
       above (`f16`, `bf16`, ... `c64`); ..."
    """
    m = re.search(
        r"scalar dtype set MUST be \*\*exactly\*\* the\s+([a-z-]+)\s+tokens"
        r"[^(]*\((.*?)\)",
        classify_text, re.S)
    if not m:
        return None, None
    return backtick_tokens(m.group(2)), m.group(1)


def markdown_tables(text):
    """Yield each markdown table as a list of rows, each row a list of cell strings."""
    rows, in_table = [], False
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("|") and s.endswith("|"):
            in_table = True
            rows.append([c.strip() for c in s.strip("|").split("|")])
        elif in_table:
            yield rows
            rows, in_table = [], False
    if in_table:
        yield rows


def full_dtype_tables(text, authoritative):
    """Every markdown table whose first column is a FULL dtype set (≥15 tokens).

    A subset table (e.g. a 3-row op-domain table) never clears the threshold, so
    it is not checked against the whole set.
    """
    auth = set(authoritative)
    for rows in markdown_tables(text):
        col0 = []
        for r in rows:
            if not r:
                continue
            cell = r[0].strip().strip("`").strip()
            toks = backtick_tokens(r[0]) or ([cell] if _is_dtype_token(cell) else [])
            if toks:
                col0.append(toks[0])
        hits = [t for t in col0 if t in auth]
        if len(hits) >= 15:
            yield col0


def inline_anchor_lists(text):
    """FULL dtype inline lists, by their specific introducing phrases.

    Two sites qualify — and only these, so subset lists are never caught:
      * the Ops §2.8 shared anchor:  "the **dtype** tokens (`...`)"
      * (the §6.1-0001 parenthetical is the authoritative source, checked apart)
    """
    for m in re.finditer(r"\*\*dtype\*\* tokens?\s*\((.*?)\)", text, re.S):
        yield "dtype-anchor", backtick_tokens(m.group(1))


def check(spec_dir):
    def read(stem):
        p = os.path.join(spec_dir, stem + ".md")
        return open(p, encoding="utf-8").read() if os.path.exists(p) else ""

    classify, ops = read("classify"), read("ops")
    violations = []

    auth, count_word = authoritative_set(classify)
    if not auth:
        return ["could not find the KISS-CLASSIFY-6.1-0001 authoritative dtype list"]
    auth_set = set(auth)

    # (3) the declared count word must equal the actual size
    if count_word in NUMBER_WORDS and NUMBER_WORDS[count_word] != len(auth):
        violations.append(
            f"§6.1-0001 says '{count_word}' ({NUMBER_WORDS[count_word]}) but lists "
            f"{len(auth)} tokens")
    if len(auth) != len(set(auth)):
        dup = [t for t in auth if auth.count(t) > 1]
        violations.append(f"§6.1-0001 list has duplicate token(s): {sorted(set(dup))}")

    def diff(where, got):
        g = set(got)
        if g != auth_set:
            miss = sorted(auth_set - g)
            extra = sorted(g - auth_set)
            parts = []
            if miss:
                parts.append(f"missing {miss}")
            if extra:
                parts.append(f"unexpected {extra}")
            violations.append(f"{where}: {'; '.join(parts)} (vs §6.1-0001 owner)")

    # (1) full dtype tables in classify.md and ops.md
    for stem, text in (("classify", classify), ("ops", ops)):
        for i, col0 in enumerate(full_dtype_tables(text, auth), 1):
            diff(f"{stem}.md full dtype table #{i}", col0)

    # (2) full dtype inline anchor lists (Ops §2.8 today)
    for kind, toks in inline_anchor_lists(ops):
        diff(f"ops.md {kind} inline list", toks)
    for kind, toks in inline_anchor_lists(classify):
        diff(f"classify.md {kind} inline list", toks)

    # (4) the determinism/fidelity enum — owned by KISS-OPS-6.0-0001, spelled
    # verbatim, imported never re-forked (umbrella §2.1, §3.3). Every brace-form
    # of the enum across the suite must carry EXACTLY the canonical member set.
    violations += check_determinism_enum(spec_dir)

    return violations, auth


# The canonical determinism/fidelity enum owned by KISS-OPS-6.0-0001.
DETERMINISM_MEMBERS = {"exact-byte", "ULP/tolerance", "order-invariant/nondeterministic"}


def check_determinism_enum(spec_dir):
    """Every `{...}` enum form containing `exact-byte`, anywhere in the suite, MUST
    carry exactly the canonical member set — this catches a downstream sub-standard
    re-forking the enum (adding/renaming a class) instead of importing it, which
    would silently split the comparator vocabulary the whole DAG shares."""
    out = []
    owner_seen = False
    for stem in SPECS:
        p = os.path.join(spec_dir, stem + ".md")
        if not os.path.exists(p):
            continue
        text = open(p, encoding="utf-8").read()
        for m in re.finditer(r"\{[^{}]*exact-byte[^{}]*\}", text):
            members = {t.strip() for t in re.split(r"[,\s]*,[,\s]*",
                       " ".join(m.group(0)[1:-1].split())) if t.strip()}
            if stem == "ops":
                owner_seen = True
            if members != DETERMINISM_MEMBERS:
                miss = sorted(DETERMINISM_MEMBERS - members)
                extra = sorted(members - DETERMINISM_MEMBERS)
                parts = []
                if miss:
                    parts.append(f"missing {miss}")
                if extra:
                    parts.append(f"unexpected {extra}")
                out.append(f"{stem}.md determinism enum re-forked: {'; '.join(parts)} "
                           f"(vs KISS-OPS-6.0-0001 owner)")
    if not owner_seen:
        out.append("KISS-OPS-6.0-0001 owner: no canonical determinism enum found in ops.md")
    return out


# The normative clauses this lint ENFORCES: a violation of each fails the lint,
# so the traceability gate may count them lint-backed (they bind the spec document
# — the closed dtype set restated in two owners — which is a linter's job, not a
# harness test). Each is (clause_id, what a violation looks like).
COVERS = [
    ("KISS-CLASSIFY-6.1-0001",
     "the closed 20-token dtype set drifts (count word or a full dtype table)"),
    ("KISS-OPS-6.16-0001",
     "the Ops §6.16 dtype layout table drops/adds a token vs the Classify owner"),
    ("KISS-OPS-6.0-0001",
     "a sub-standard re-forks the determinism/fidelity enum instead of importing it"),
]


def main():
    ap = argparse.ArgumentParser(description="KISS shared-enumeration consistency lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note), "
                         "for tools/kiss_trace.py to count as lint-backed")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(here), "spec")

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    result = check(spec_dir)
    if isinstance(result, list):  # fatal
        print("KISS table lint — FATAL")
        for v in result:
            print(f"  - {v}")
        return 1
    violations, auth = result

    print("KISS shared-enumeration lint  —  " + spec_dir)
    print("=" * 68)
    print(f"  dtype set (owner KISS-CLASSIFY-6.1-0001): {len(auth)} tokens")
    print(f"    {' '.join(auth)}")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} enumeration(s) disagree with the owner:")
        for v in violations:
            print(f"      - {v}")
        print("-" * 68)
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  every full dtype table and anchor list agrees with the owner.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
