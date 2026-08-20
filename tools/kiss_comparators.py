#!/usr/bin/env python3
"""Comparator-blindness lint (KISS-CONFORM-6.8-0012).

A check that normalizes away a difference cannot see the difference it normalizes.
That is true of every CORRECT normalizing comparator -- structural op-DAG equality
resolves to the primitive floor on purpose -- so the hazard is never that a
comparator normalizes. It is that nothing records WHICH dimensions it is blind to,
so a clause can cite it as backing for an obligation it structurally cannot observe,
and the test passes forever.

This lint enforces the DECLARATION half: every clause defining a comparison relation
carries a `Normalizes:` enumeration, or says it normalizes nothing.

WHY THE DETECTOR IS OVER-BROAD AND THEN EXPLICITLY EXCLUDED, rather than a
heuristic that recognizes "defining" clauses. A heuristic that decides which clauses
are comparator DEFINITIONS would silently exempt any clause it failed to recognize --
and a new comparator arriving unrecognized is exactly the case this lint exists for.
So: EVERY clause mentioning a comparison relation is in scope, and each is either
declared or listed below with a reason. A new one matches, is in neither set, and
FAILS CLOSED. The cost is maintaining SELECTS_ONLY by hand; the cost of the
alternative is a comparator nobody classified.

Exit 0 clean, 1 on a violation.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
DOC = os.path.join(os.path.dirname(HERE), "spec", "conform.md")

RE_CLAUSE = re.compile(r"^- \*\*(KISS-CONFORM-[0-9.]+-[0-9]{4}[a-z]?)\*\*", re.M)
# Over-broad on purpose: any clause body naming a comparison relation.
RE_INSCOPE = re.compile(r"\bcomparator\b|\bbyte-match(?:es|ed)?\b", re.I)
RE_DECLARED = re.compile(r"\*\*`?Normalizes:`?\*\*")

# Clauses that SELECT, APPLY, or CONSTRAIN a comparator without defining one. Each
# needs a reason, because "it isn't a definition" is the judgement this lint refuses
# to make for itself.
SELECTS_ONLY = {
    # -- selection / application rules: they say WHICH comparator applies, never what one IS.
    "KISS-CONFORM-6.0-0002": "selection is by declared class, not by the test author; defines none",
    "KISS-CONFORM-6.0-0003": "declares structural artifacts determinism-class-bearing; defines none",
    "KISS-CONFORM-6.3-0002": "matrix metadata copied from the sidecar; names a class, defines none",
    "KISS-CONFORM-6.4-0001": "golden byte-vectors APPLY the byte-exact comparator of 6.8-0001",
    "KISS-CONFORM-6.5-0001": "oracle-differential harness applies the selected comparator",
    "KISS-CONFORM-6.8-0006": "selection rule (declared class -> comparator); defines none",
    "KISS-CONFORM-6.8-0008": "totality/unambiguity of selection; defines none",
    "KISS-CONFORM-6.8-0009": "restricts which comparator is admissible for POD wire fields",
    "KISS-CONFORM-6.8-0011": "multi-output application rule; defines no relation",
    "KISS-CONFORM-6.8-0012": "the declaration obligation itself",
    "KISS-CONFORM-6.8-0013": "the exhibition obligation; requires a demonstration, defines no relation",
    "KISS-CONFORM-6.9-0002": "requires the tier-1 comparator BE the structural one",
    "KISS-CONFORM-6.9-0003": "applies the structural comparator to a named round-trip",
    "KISS-CONFORM-6.12-0002": "consumer-verify runs under the declared-precision comparator",
    "KISS-CONFORM-6.13-0001": "applies the byte-exact comparator to a sub-standard's clauses",
    "KISS-CONFORM-6.13-0003": "applies the byte-exact comparator; defines none",
    "KISS-CONFORM-6.13-0005": "requires the three comparators be IMPLEMENTED; defines none",
    "KISS-CONFORM-6.13-0006b": "selects the differential comparator from the advertised class",
    "KISS-CONFORM-6.13-0007": "requires the split comparator be implemented; 6.8-0005 defines it",
    "KISS-CONFORM-6.13-0013": "selects from the provided kernel's contract determinism class",
    "KISS-CONFORM-6.13-0017": "OWNS and applies the structural comparator; 6.9-0001 defines it",
    "KISS-CONFORM-7.3-0001": "forbids adding a comparator to the enum; defines none",
    "KISS-CONFORM-8-0007": "reference implementation runs the same suite; defines none",
    "KISS-CONFORM-8-0010": "freeze-gate wording referencing comparator selection; defines none",
}


# --- §6.8-0012 completeness: every bucketed component of the key must be declared -------
#
# A `Normalizes:` enumeration that OMITS a dimension is worse than one that names it: a reader
# checks the list, does not find `extent`, concludes the key discriminates extents, and may cite
# the admissibility match for an obligation that cannot fail above the bucket ceiling. The
# declaration was in fact short by SIX dimensions when first written -- true as far as it went.
#
# The alphabet a bucket collapses onto is declared by `code_enum!` in the codec, so the CODEC is
# the population and the CLAUSE is what must mention it. Presence is tested by TOKEN CODE, not by
# type name: a declaration that paraphrases the dimension without naming a token it collapses has
# not told the reader which values are indistinguishable.
KEY_CODEC = os.path.join(os.path.dirname(HERE), "conformance", "src", "structure_key.rs")
RE_CODE_ENUM = re.compile(r"code_enum!\(\s*(\w+)\s*\{([^}]*)\}", re.S)
RE_CODE_TOK = re.compile(r'=\s*"([^"]+)"')

# Alphabets that are NOT a bucketing of a continuum, each with the reason. A bucket collapses a
# RANGE onto one token; an enum of declared alternatives does not, and forcing it into the
# declaration would claim a blindness the key does not have.
NON_BUCKET = {
    "MathPrecision": "declared attribute (st/rm), not a bucketing of any continuum",
}

# Index width is derived as a &'static str pair rather than a `code_enum!`, so the scan above
# cannot see it. DETECTED FROM THE SOURCE, not hardcoded: a hardcoded entry would be asserted
# for a fixture codec that has no such derivation, and would keep being asserted after a rename.
# KNOWN LIMIT: a future alphabet that is neither a code_enum! nor this pair fails OPEN.
IX_TOKENS = ("ix32", "ix64")


def key_alphabets(path=KEY_CODEC):
    """{name: (token, ...)} for every bounded alphabet the structure_key carries."""
    out = {}
    try:
        with open(path, encoding="utf-8") as fh:
            src = fh.read()
    except OSError:
        return None
    for m in RE_CODE_ENUM.finditer(src):
        out[m.group(1)] = tuple(RE_CODE_TOK.findall(m.group(2)))
    if all(f'"{t}"' in src for t in IX_TOKENS):
        out["index width"] = IX_TOKENS
    return out


def undeclared_buckets(clause_body, path=KEY_CODEC):
    """Alphabets whose tokens appear nowhere in the clause and are not excused."""
    alpha = key_alphabets(path)
    if alpha is None:
        return None
    missing = []
    for name, toks in sorted(alpha.items()):
        if name in NON_BUCKET or not toks:
            continue
        if not any(f"`{t}`" in clause_body for t in toks):
            missing.append(name)
    return missing


def clause_bodies(text):
    """(clause_id, body) for every clause, body running to the next clause."""
    marks = [(m.group(1), m.start()) for m in RE_CLAUSE.finditer(text)]
    for i, (cid, start) in enumerate(marks):
        end = marks[i + 1][1] if i + 1 < len(marks) else len(text)
        yield cid, text[start:end]


def scan(path=DOC):
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    declared, undeclared, excluded = [], [], []
    for cid, body in clause_bodies(text):
        if not RE_INSCOPE.search(body):
            continue
        if RE_DECLARED.search(body):
            declared.append(cid)
        elif cid in SELECTS_ONLY:
            excluded.append(cid)
        else:
            undeclared.append(cid)
    stale = sorted(set(SELECTS_ONLY) - set(excluded) - set(declared))
    return declared, undeclared, excluded, stale


def main():
    # `--emit-coverage` is how kiss_trace.py credits a `lint:` ledger category: the
    # lint ASSERTS the clauses it enforces, and because the tool is actually run to
    # collect this, the label cannot outrun the enforcement. Emitting a clause here
    # while the lint cannot fail on it would be the exact defect §6.8-0012 names.
    if "--emit-coverage" in sys.argv:
        print("KISS-CONFORM-6.8-0012	every clause naming a comparison relation declares "
              "what it normalizes, or is explicitly recorded as defining none")
        return 0
    declared, undeclared, excluded, stale = scan()
    # Completeness of the admissibility match's own declaration (#274).
    with open(DOC, encoding="utf-8") as fh:
        _doc = fh.read()
    _body = next((b for c, b in clause_bodies(_doc) if c == "KISS-CONFORM-6.8-0012"), "")
    buckets = undeclared_buckets(_body)
    print("KISS-Conform comparator-blindness lint  -  §6.8-0012")
    print("=" * 68)
    print(f"  {len(declared):3d} clause(s) declare what they normalize")
    print(f"  {len(excluded):3d} select/apply a comparator without defining one (explicit)")
    print(f"  {len(undeclared):3d} name a comparison relation and declare NOTHING")
    bad = False
    if undeclared:
        bad = True
        print("-" * 68)
        print("  VIOLATION: a comparison relation with no `Normalizes:` enumeration.")
        print("  Add one, or record it in SELECTS_ONLY with the reason it defines none:")
        for cid in undeclared:
            print(f"          - {cid}")
    if stale:
        bad = True
        print("-" * 68)
        print("  STALE EXCLUSION: listed in SELECTS_ONLY but no longer in scope —")
        print("  the clause was renamed, retired, or no longer names a comparison relation.")
        print("  A stale exclusion silently exempts nothing today and the WRONG clause tomorrow:")
        for cid in stale:
            print(f"          - {cid}")
    if buckets is None:
        bad = True
        print("-" * 68)
        print("  UNVERIFIED: the key codec could not be read, so the admissibility match's")
        print("  declaration could not be checked for completeness. Refusing the green rather")
        print("  than passing an unchecked condition.")
    elif buckets:
        bad = True
        print("-" * 68)
        print("  INCOMPLETE DECLARATION: §6.8-0012 names the admissibility match but omits")
        print("  bucketed component(s) of the key. A reader checks the list, does not find the")
        print("  dimension, and may cite the match for an obligation that cannot fail:")
        for b in buckets:
            print(f"          - {b}")
        print("  Name a token the bucket collapses onto, or record it in NON_BUCKET with why.")
    if not declared:
        bad = True
        print("-" * 68)
        print("  VACUITY: zero declarations found. The pattern matched nothing, so a clean")
        print("  run here would mean the lint is broken, not that the document is.")
    print("-" * 68)
    print("  RESULT: VIOLATIONS FOUND" if bad else "  RESULT: CLEAN")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
