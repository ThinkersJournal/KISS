#!/usr/bin/env python3
"""Convention 16(e) — a bare `§N.M` cite is ambiguous outside the document defining it.

16(e) says this is the one convention that is MECHANICALLY CHECKABLE and can have a
detector rather than only a rule. It went four days without one, and two people who knew
the rule broke it in the interval — `§6.8-0008` is a FOUR-way ambiguity (CLASSIFY /
CONFORM / CONTRACT / GRAMMAR) and `§6.7` a SEVEN-way one. The ambiguity is invisible from
the inside: the author resolves it unconsciously on every re-read, so it costs them
nothing and costs every other reader the lookup. A class vigilance cannot close gets a
detector.

WHAT IS A VIOLATION: a `§`-cite naming a section the CONTAINING document does not define,
with nothing binding it to a sub-standard. A bare `§6.7` inside `conform.md` meaning
conform.md's own §6.7 is CORRECT and must not flag.

WHAT IS QUALIFIED, and all of it is SYNTACTIC — no distance thresholds. A 60-char
proximity window was measured and rejected: a threshold nobody re-derives is a parameter
that silently stops meaning anything.

    KISS-Ops §6.4                adjacency
    KISS-Ops (§6.4)              parenthetical apposition
    KISS-OPS **§6.19-0001**      markdown emphasis between the two
    KISS-Emit's §6.7-0008        possessive
    KISS-OPS\n  §6.19-0005       the name ends the previous line
    KISS-Synth (§1, §2.4)        COORDINATION: one qualifier distributed over a list

The last three were NOT in the ruling that specified this tool; they were found by
measuring the corpus against the rule before building to it. A qualification rule written
to the two forms in view would have flagged 138 correct cites — a rule whose accuracy is a
function of a property of the examples that shaped it, which is the same defect as a regex
that stops at two dotted levels because every example in view had two.

COORDINATION IS MODELLED NARROWLY AND ON PURPOSE. A bare cite qualifies only inside the
same DELIMITED group as a qualified one — the same parenthetical, or joined by an explicit
coordinator with no sentence boundary between. Delimited by punctuation, never by distance.
An over-broad coordination model is worse than a narrow one that misses: over-qualifying is
SILENT and hides the real ambiguities this exists to find, while a miss lands on the
judgement list where a human sees it. When in doubt, list it.

QUOTATION IS EXEMPT, and getting this wrong makes the lint actively harmful. #310 requires
a paraphrase to preserve the other party's identifiers, so a bare cite inside quoted or
attributed text is CORRECT — rewriting it would trade an unresolvable reference for an
unreliable quotation. Measured at 7 instances, not a corner case. A lint that flags the
correct fix teaches everyone to ignore it.

Exit 0 clean, 1 on a violation. The JUDGEMENT list never gates: those are cases the
syntactic model cannot decide, and a human decides them one at a time.

Run: python tools/kiss_scoped_cites.py [--list-judgement]
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)

DOCS = ["spec/umbrella.md", "spec/announce.md", "spec/classify.md", "spec/ops.md",
        "spec/grammar.md", "spec/contract.md", "spec/synth.md", "spec/consume.md",
        "spec/emit.md", "spec/conform.md", "spec/namespaces/cuda.md",
        "spec/namespaces/vulkan.md", "CONTRIBUTING.md"]

SUBS = ["umbrella", "announce", "classify", "ops", "grammar", "contract", "synth",
        "consume", "emit", "conform"]
NAME = r"(?:KISS-[A-Za-z]+|" + "|".join(SUBS) + r")"

RE_CITE = re.compile(r"§(?P<sec>\d+(?:\.\d+)+)(?P<clause>-\d+)?")
RE_HEADING = re.compile(r"^#{2,4}\s+(\d+(?:\.\d+)*)", re.M)
RE_OWN_CLAUSE = re.compile(r"KISS-[A-Z]+-(\d+(?:\.\d+)*)-\d+")
# `NAME('s)? <empty tokens> §` — markup, spaces and newlines carry no meaning here.
RE_QUAL_ADJ = re.compile(NAME + r"(?:'s)?[\s*_`>]*$", re.I)
# `NAME (§` — apposition, the name immediately parenthesising the cite. The bracket is
# the delimiter; there is deliberately NO character budget between name and `(`. An
# earlier draft allowed 60 chars there, which is the same distance threshold the 60-char
# proximity window was rejected for, smuggled back in as a regex quantifier.
RE_QUAL_APPO = re.compile(NAME + r"(?:'s)?[\s*_`]*\([\s*_`]*$", re.I)
RE_COORD = re.compile(r"(?:\band\b|\bor\b|,|;|/|\+)[\s*_`]*$", re.I)
RE_SENT_END = re.compile(r"[.!?]\s")
RE_NAME_ANY = re.compile(NAME, re.I)


def defined_sections(text):
    """Sections this document defines — headings plus its own clause IDs."""
    return set(RE_HEADING.findall(text)) | set(RE_OWN_CLAUSE.findall(text))


def defines(sec, secs):
    """True if `secs` covers `sec` or a SUB-SECTION ancestor — §3.1.2 lives inside §3.1.

    Ancestors start at TWO components. A one-component match would let `## 6.
    Specification` — a heading every sub-standard has — claim every `§6.x` cite in the
    suite as a self-reference, and the detector would report 0 violations against a
    corpus measured to hold 31. That is the over-qualifying direction, and it is SILENT:
    a lint that qualifies everything looks exactly like a clean document.
    """
    parts = sec.split(".")
    if len(parts) < 2:
        return False
    return any(".".join(parts[:i]) in secs for i in range(2, len(parts) + 1))


def _clause_scope(pre):
    """The text back to the nearest sentence boundary — the coordination window.

    Punctuation, never distance. A coordinator cannot reach across a full stop.
    """
    ends = list(RE_SENT_END.finditer(pre))
    return pre[ends[-1].end():] if ends else pre


def classify_cite(text, pos, secs):
    """One of: self / qualified / quoted / judgement / violation."""
    pre = text[:pos]
    line_start = pre.rfind("\n") + 1
    line = text[line_start:text.find("\n", pos) if text.find("\n", pos) != -1 else len(text)]
    before_on_line = text[line_start:pos]

    if line.lstrip().startswith(">") or before_on_line.count('"') % 2 == 1:
        return "quoted"
    if RE_QUAL_ADJ.search(pre) or RE_QUAL_APPO.search(pre):
        return "qualified"
    scope = _clause_scope(pre)
    if RE_COORD.search(scope) and RE_NAME_ANY.search(scope):
        # a coordinator joining this cite to a qualified one in the SAME clause
        return "qualified"
    if RE_NAME_ANY.search(scope):
        return "judgement"
    return "violation"


def scan(root=ROOT):
    """(violations, judgement) as [(doc, line, cite)] each."""
    texts = {}
    for d in DOCS:
        p = os.path.join(root, d.replace("/", os.sep))
        if os.path.exists(p):
            with open(p, encoding="utf-8") as fh:
                texts[d] = fh.read()
    violations, judgement = [], []
    for d, text in texts.items():
        secs = defined_sections(text)
        for m in RE_CITE.finditer(text):
            if defines(m.group("sec"), secs):
                continue
            verdict = classify_cite(text, m.start(), secs)
            if verdict in ("qualified", "quoted", "self"):
                continue
            row = (d, text[:m.start()].count("\n") + 1,
                   "§" + m.group("sec") + (m.group("clause") or ""))
            (violations if verdict == "violation" else judgement).append(row)
    return violations, judgement


def main():
    violations, judgement = scan()
    print("KISS convention 16(e) — bare cross-document §-cites")
    print("=" * 68)
    print(f"  {len(violations):4d} violation(s)")
    print(f"  {len(judgement):4d} on the judgement list (reported, never gated)")
    if violations:
        print("-" * 68)
        print("  A bare cite naming a section this document does not define. Qualify it")
        print("  (`KISS-Ops §6.4`) — the reader cannot resolve it and the author cannot see")
        print("  that they cannot:")
        for d, line, cite in violations:
            print(f"          {d}:{line}  {cite}")
    if judgement and "--list-judgement" in sys.argv:
        print("-" * 68)
        print("  JUDGEMENT — a sub-standard is named in the clause but binds no cite")
        print("  syntactically. Decided one at a time; the model deliberately does not")
        print("  stretch to cover these, because an over-broad rule goes SILENT on the")
        print("  ambiguities it exists to find:")
        for d, line, cite in judgement:
            print(f"          {d}:{line}  {cite}")
    print("-" * 68)
    print("  RESULT: VIOLATIONS FOUND" if violations else "  RESULT: CLEAN")
    return 1 if violations else 0


if __name__ == "__main__":
    sys.exit(main())
