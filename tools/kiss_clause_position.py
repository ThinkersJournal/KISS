#!/usr/bin/env python3
"""
kiss_clause_position.py — every clause DEFINITION sits under its own section heading.

A *document* lint, and the axis no existing instrument reads. kiss_clause_form checks a clause is
readable as one sentence; kiss_trace checks a clause MAPS to a test; kiss_tables checks re-listed
enumerations agree. NONE checks a clause is in the right PLACE.

WHY THIS IS NOT A STYLE RULE. On 2026-09-06 a normative clause (`KISS-CONFORM-8-0011`) landed
AFTER the document's closing italic summary, past Appendix E and Appendix F, and SIX gates passed:

    ratchet 387/33/501 CLEAN · README binding CLEAN
    clause_form PASS · scoped_cites PASS · vocab PASS · cites PASS

The traceability matrix stayed order-correct the whole time, because it is a SEPARATE list and the
one instrument that reads both the body and the matrix compares MEMBERSHIP, never POSITION. A
clause outside its section is normative text a reader will not find under the heading that governs
it, and a reader who cannot find it under §8 does not conclude it is misfiled — they conclude it is
absent. #486.

THE CHECK. Track the current numbered section from the most recent `##`/`###` `<N(.M)>` heading. A
non-numbered `##` heading (an Appendix, the closing summary) LEAVES the numbered sections
(section = None). A `####` sub-heading is a sub-part and does not change the section; the `#` title
line does not either. Every clause definition `- **KISS-<STD>-<sec>-<nnnn>** —` MUST sit where the
current section is a PREFIX of `<sec>` — i.e. under its own `### <sec>` heading, or under a parent
`## <sec-prefix>` heading when the sub-section has no heading of its own (a clause `8.1-0001` under
`## 8` is correctly placed). A clause under a *sibling* section (`6.6-0001` under §6.5), or under
none — past the appendices/summary — is flagged.
"""
import pathlib
import re
import sys

HEADING = re.compile(r"^(#{1,6})\s+(.*)$")
# `\.\d+)*` — ARBITRARY depth, not two levels: a heading `### 6.19.1` or a clause id
# `KISS-CONFORM-1.1.1-0001` must not be silently missed or truncated to `6.19` (a false absence
# in the census / a mis-tracked section). Today's clause ids are two-level and the deeper `6.19.x`
# headings are `####` sub-parts (not section-tracked), but the regex must not go blind if that changes.
SECNUM = re.compile(r"^(?:§\s*)?(\d+(?:\.\d+)*)\b")
# A clause DEFINITION: a bullet whose first token is the bolded id, followed by an em/en dash.
# Excludes matrix rows (`| KISS-… |`) and prose cross-refs (`§6.6-0007`, bare `KISS-…`).
CLAUSE_DEF = re.compile(r"^-\s+\*\*(KISS-[A-Z]+-(\d+(?:\.\d+)*)-\d+[a-z]?)\*\*\s*[—–-]")


def _is_under(current, sec):
    """True iff a clause of section `sec` is correctly placed under heading section `current`:
    `current` must be a dotted-component prefix of `sec` (equal counts). None is never a prefix."""
    if current is None:
        return False
    cur_parts = current.split(".")
    return sec.split(".")[: len(cur_parts)] == cur_parts


def scan(path):
    """Return (offenders, total). `offenders` is a list of (line_no, clause_id, sec, current)."""
    cur = None  # the current numbered section, or None once we leave the numbered body
    offenders = []
    total = 0
    for i, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        h = HEADING.match(line)
        if h:
            level = len(h.group(1))
            if 2 <= level <= 3:  # ## / ### set the section; # is the title, #### a sub-part
                m = SECNUM.match(h.group(2).strip())
                cur = m.group(1) if m else None
            continue
        c = CLAUSE_DEF.match(line)
        if c:
            total += 1
            cid, sec = c.group(1), c.group(2)
            if not _is_under(cur, sec):
                offenders.append((i, cid, sec, cur))
    return offenders, total


def main():
    spec = pathlib.Path(__file__).resolve().parent.parent / "spec"
    print("-" * 68)
    print("  KISS clause-position lint — a clause MUST sit under its own section heading")
    print("-" * 68)
    grand_total = bad = 0
    for md in sorted(spec.rglob("*.md")):
        offenders, total = scan(md)
        grand_total += total
        for ln, cid, sec, cur in offenders:
            bad += 1
            where = f"§{cur}" if cur else "no numbered section (past an appendix/summary)"
            print(f"      [MISPLACED] {md.name}:{ln}  {cid}")
            print(f"                  belongs under §{sec}; sits under {where}")
    print("-" * 68)
    # A lint reporting CLEAN on an empty population cannot tell "checked and fine" from "found
    # nothing to check" — which is what happens if spec/ moves, the glob breaks, or the clause
    # bullet form changes. Exit 2 so a non-measurement is never read as a pass (see kiss_clause_form).
    if grand_total == 0:
        print(f"  NO clause definitions found under {spec}.")
        print("  Either the spec moved, the glob is wrong, or the clause bullet form changed.")
        print("  RESULT: COULD NOT MEASURE")
        return 2
    if bad:
        print(f"  {bad} of {grand_total} clause definitions sit outside their section.")
        print("  Move each under the `## <sec>` / `### <sec>` heading its id names.")
        print("  RESULT: CLAUSE POSITION BROKEN")
        return 1
    print(f"  {grand_total} clause definitions, each under its own section heading.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
