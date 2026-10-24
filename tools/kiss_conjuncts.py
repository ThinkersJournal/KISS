"""KISS#387 (a): the per-conjunct trace behind umbrella §5.3 condition 3.

`kiss_trace.py --freeze-ready` asks whether a clause HAS a backing. It cannot ask whether the
backing demonstrates the WHOLE clause: a sampled audit found about half of even well-evidenced
pointers cover only part of their clause (KISS#387). This module closes that for the clauses
that are conjunctions.

`conformance/CONJUNCTS.tsv` lists, for each multi-conjunct clause, one row per conjunct:

    <clause id> TAB <label> TAB <one-line statement of the conjunct>

A test cites a conjunct in a backing comment, `// Backs: KISS-X-6.8-0005 [zero-sign-re]` (several
labels: `[a, b]`). A listed clause is TRACED only when EVERY one of its labels is cited by some
test. A bare `// Backs: KISS-X` still backs the clause for the existing floor; it simply cannot
discharge a conjunct.

Limit (stated, not hidden): which clauses are conjunctions, and how they split, is editorial.
The ledger proves each LISTED conjunct is cited; it cannot prove the list is complete, nor that
a citing test discriminates its conjunct (that stays a mutation-audit question, KISS#278).
"""
import os
import re

CLAUSE_ID = r"KISS-[A-Z]+-\d+(?:\.\d+)?-\d{4}[a-z]?"
LABEL = r"[a-z][a-z0-9-]*"
RE_KEYWORD = re.compile(r"\b(?:Backs|Enforces)\b", re.I)
RE_CONJ_CITE = re.compile(
    r"(" + CLAUSE_ID + r")\s*\[(" + LABEL + r"(?:\s*,\s*" + LABEL + r")*)\]")
RE_LABEL = re.compile(r"^" + LABEL + r"$")
RE_CLAUSE = re.compile(r"^" + CLAUSE_ID + r"$")


def conjunct_cites(scope):
    """{clause id: {label, ...}} cited by `Backs:`/`Enforces` COMMENT lines in a test's scope.
    A bracketed label on any other line (a string, a code line) is not a citation."""
    cites = {}
    for line in scope.splitlines():
        if not line.lstrip().startswith("//") or not RE_KEYWORD.search(line):
            continue
        for cid, labels in RE_CONJ_CITE.findall(line):
            cites.setdefault(cid, set()).update(x.strip() for x in labels.split(","))
    return cites


def load(text):
    """Parse a CONJUNCTS.tsv body. Returns ({clause: {label: statement}}, [problem, ...])."""
    rows, problems = {}, []
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip("\r")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        cols = line.split("\t")
        if len(cols) != 3 or not all(c.strip() for c in cols):
            problems.append(f"CONJUNCTS.tsv:{n}: want `clause<TAB>label<TAB>statement`")
            continue
        cid, label, stmt = (c.strip() for c in cols)
        if not RE_CLAUSE.match(cid) or not RE_LABEL.match(label):
            problems.append(f"CONJUNCTS.tsv:{n}: bad clause id or label `{cid}` `{label}`")
        elif label in rows.get(cid, {}):
            problems.append(f"CONJUNCTS.tsv:{n}: duplicate label `{label}` for {cid}")
        else:
            rows.setdefault(cid, {})[label] = stmt
    for cid, labels in sorted(rows.items()):
        if len(labels) < 2:
            problems.append(f"{cid}: one conjunct is not a conjunction; drop the row")
    return rows, problems


def structural_problems(ledger, tests, clause_ids):
    """Faults that are wrong whenever they exist (not only at a freeze): a ledger clause that
    is not a live clause, and a citation of a label the ledger does not list for that clause."""
    out = [f"{cid}: listed in CONJUNCTS.tsv but is not a clause of the spec"
           for cid in sorted(ledger) if cid not in clause_ids]
    for name, t in sorted(tests.items()):
        for cid, labels in sorted(t.get("conj_cites", {}).items()):
            if cid not in ledger:
                out.append(f"{name}: cites a conjunct of {cid}, which CONJUNCTS.tsv does not list")
                continue
            out += [f"{name}: cites unknown conjunct [{lb}] of {cid}"
                    for lb in sorted(labels - set(ledger[cid]))]
    return out


def uncited(ledger, tests, sub_of=None, sub=None):
    """{clause: [label, ...]} listed conjuncts no test cites; `sub` limits to one sub-standard."""
    seen = {}
    for t in tests.values():
        for cid, labels in t.get("conj_cites", {}).items():
            seen.setdefault(cid, set()).update(labels)
    out = {}
    for cid, labels in sorted(ledger.items()):
        if sub is not None and sub_of(cid) != sub:
            continue
        miss = sorted(set(labels) - seen.get(cid, set()))
        if miss:
            out[cid] = miss
    return out


def read_conjuncts(conf_dir):
    """(ledger, problems) from conformance/CONJUNCTS.tsv; a missing file is an empty ledger."""
    path = os.path.join(conf_dir, "CONJUNCTS.tsv")
    if not os.path.isfile(path):
        return {}, []
    with open(path, encoding="utf-8") as fh:
        return load(fh.read())
