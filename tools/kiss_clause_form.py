#!/usr/bin/env python3
"""
kiss_clause_form.py — the KISS clause-shape lint.

A *document* lint. Where kiss_trace checks that a clause maps to an executable
test and kiss_tables checks that re-listed enumerations agree, this checks that a
clause's PROSE IS READABLE AS ONE SENTENCE — specifically that a metadata block
(`**Normalizes:**`) is not wedged inside the requirement it annotates.

WHY THIS IS NOT A STYLE RULE. On 2026-09-02 every one of conform.md's six
`Normalizes:` lines sat inside its clause's sentence, and three split a compound
mid-phrase:

    ...transitively contains a transcendental
    **Normalizes:** numeric difference within the op's declared ULP bound.
    atom, and KISS-Conform MUST NOT claim cross-language numeric identity...

Two consequences, both observed rather than predicted:

  1. A grep for "transitively contains a transcendental atom" does not return the
     clause. A phrase split across a metadata line is not findable as a string, so
     a spec search reports a FALSE ABSENCE.

  2. Two readers independently quoted KISS-CONFORM-6.8-0001 with the interrupting
     line silently removed, neither noticing they had reassembled it, and a
     normative ruling was made partly on one of those reassemblies. A reader who
     cannot parse a clause ASKS; a reader who unconsciously REPAIRS one PROCEEDS,
     and the repair is invisible to everyone downstream including the repairer.

Fixed by #388 (two clauses) and #391 (the other four). This lint exists so the
convention cannot re-establish itself: the placement was uniform across all six,
which means it was the house style rather than six accidents.

THE CHECK. A `**Normalizes:**` line is mid-sentence when the line above it does
not terminate (no `.` `:` `*` or backtick) AND the line below it continues in
lowercase. Both conditions are required: a metadata line following a complete
sentence is correctly placed, and one followed by a new bullet or a heading ends
the clause.
"""
import pathlib
import sys

META = "**Normalizes:**"
TERMINATORS = (".", ":", "*", "`")


def offenders(path):
    """Yield (line_no, prev_tail, next_head) for each mid-sentence metadata line."""
    lines = path.read_text(encoding="utf-8").splitlines()
    for i, line in enumerate(lines):
        if META not in line or i == 0:
            continue
        prev = lines[i - 1].rstrip()
        nxt = lines[i + 1].strip() if i + 1 < len(lines) else ""
        if prev and not prev.endswith(TERMINATORS) and nxt[:1].islower():
            yield i + 1, prev[-52:], nxt[:52]


def main():
    spec = pathlib.Path(__file__).resolve().parent.parent / "spec"
    print("-" * 68)
    print("  KISS clause-form lint — a metadata block MUST NOT split a requirement")
    print("-" * 68)
    total = bad = 0
    for md in sorted(spec.rglob("*.md")):
        seen = sum(1 for l in md.read_text(encoding="utf-8").splitlines() if META in l)
        total += seen
        for ln, prev, nxt in offenders(md):
            bad += 1
            print(f"      [SPLIT] {md.name}:{ln}")
            print(f"              ...{prev}")
            print(f"              -> {nxt}...")
    print("-" * 68)
    if bad:
        print(f"  {bad} of {total} `Normalizes:` blocks interrupt their own requirement.")
        print("  Move each to sit after the sentence it annotates (see #389).")
        print("  RESULT: CLAUSE FORM BROKEN")
        return 1
    print(f"  {total} `Normalizes:` blocks, none splitting a requirement.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
