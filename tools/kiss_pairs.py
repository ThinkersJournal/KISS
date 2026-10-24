# SPDX-License-Identifier: MIT OR Apache-2.0
"""KISS#240 / #239 item 3: the mechanical pair checker behind KISS-CONFORM-6.14.

For a token grammar a *coordinate* is a token field, and a *discriminating pair* is two positive
vectors whose tokens differ in field k ALONE (so the required outputs differ, 6.14-0001). Whether
one exists is decided here over the vector file, never asserted by an author (6.14-0003).

`conformance/UNDISCRIMINATED.tsv` records, per coordinate with no pair:

    <vector file> TAB <field index> TAB <category> TAB <reason>

category `undiscriminated` = no pair can be constructed, the reason says why (6.14-0002);
category `unauthored` = a pair is constructible but not yet in the vector set: recorded debt,
reported on every run (6.14-0004 wants this to fail the build; the ledger is the ratchet that
lets the existing gaps be named once and never grow).

Exit 1 when a coordinate has no pair and no ledger row, or when a ledger row is stale (the pair
now exists, or the field is out of range). Limit, stated: a pair here shows the TOKEN differs in
one field; it does not show the differing outputs came from a differing INPUT coordinate, and
the field list is editorial (the structure_key grammar, spec/classify.md section 2.8).
"""
import itertools
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
VECTORS = "conformance/corpus/structure_key_vectors.json"
LEDGER = "conformance/UNDISCRIMINATED.tsv"
FIELDS = ["schema", "cell_class", "dtype", "target", "index_width", "schedule", "rank",
          "operands", "reduce", "contraction"]
CATEGORIES = ("undiscriminated", "unauthored")


def split(token):
    """Fields of a token, padded to the full grammar (the contraction trailer is optional)."""
    f = token.split("|")
    return f + [""] * (len(FIELDS) - len(f))


def pairs(tokens):
    """{field index: [(i, j), ...]} for every vector pair differing in exactly that field."""
    rows = [split(t) for t in tokens]
    out = {k: [] for k in range(len(FIELDS))}
    for i, j in itertools.combinations(range(len(rows)), 2):
        diff = [k for k in range(len(FIELDS)) if rows[i][k] != rows[j][k]]
        if len(diff) == 1:
            out[diff[0]].append((i, j))
    return out


def load(text):
    """Parse the ledger. Returns ({(file, field): (category, reason)}, [problem, ...])."""
    rows, problems = {}, []
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip("\r")
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        cols = line.split("\t")
        if len(cols) != 4 or not cols[1].isdigit() or int(cols[1]) >= len(FIELDS):
            problems.append(f"line {n}: want <file> TAB <field 0-{len(FIELDS) - 1}> TAB <category> TAB <reason>")
        elif cols[2] not in CATEGORIES or len(cols[3].strip()) < 20:
            problems.append(f"line {n}: category must be one of {CATEGORIES} and the reason a real sentence")
        elif (cols[0], int(cols[1])) in rows:
            problems.append(f"line {n}: duplicate row for field {cols[1]}")
        else:
            rows[(cols[0], int(cols[1]))] = (cols[2], cols[3].strip())
    return rows, problems


def check(vector_file, tokens, rows):
    """Return (report lines, violation lines) for one vector file."""
    found, report, bad = pairs(tokens), [], []
    for k, name in enumerate(FIELDS):
        led = rows.get((vector_file, k))
        n = len(found[k])
        if n and led:
            bad.append(f"STALE: field {k} ({name}) has {n} pair(s) but a ledger row says {led[0]}")
        elif n:
            report.append(f"  field {k} {name}: {n} discriminating pair(s)")
        elif led:
            report.append(f"  field {k} {name}: {led[0].upper()} - {led[1]}")
        else:
            bad.append(f"MISSING: field {k} ({name}) has no discriminating pair and no ledger row (6.14-0004)")
    stale = [k for (f, k) in rows if f == vector_file and k >= len(FIELDS)]
    bad += [f"STALE: ledger names field {k}, out of range" for k in stale]
    return report, bad


def main():
    with open(os.path.join(ROOT, VECTORS), encoding="utf-8") as fh:
        tokens = [v["token"] for v in json.load(fh)["positive_vectors"]]
    with open(os.path.join(ROOT, LEDGER), encoding="utf-8") as fh:
        rows, problems = load(fh.read())
    report, bad = check(VECTORS, tokens, rows)
    print(f"KISS-CONFORM-6.14 pair check over {VECTORS} ({len(tokens)} positive vectors)")
    print("\n".join(report))
    debt = sum(1 for c, _ in rows.values() if c == "unauthored")
    print(f"  recorded unauthored debt: {debt} coordinate(s)")
    for line in problems + bad:
        print("VIOLATION: " + line)
    return 1 if problems or bad else 0


if __name__ == "__main__":
    sys.exit(main())
