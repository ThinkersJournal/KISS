#!/usr/bin/env python3
"""
kiss_ops.py — the KISS-Ops within-document op-set consistency lint.

A pure-spec document lint (no harness coupling), a sibling of tools/kiss_tables.py.
Where kiss_tables checks a shared enumeration across SEVERAL spec files and
kiss_vocab checks a spec enumeration against the reference codec, this checks the
op catalogue that KISS-Ops (ops.md) deliberately RESTATES within its own file — the
§2.7 informative catalogue, the §6.3 primitive-floor clause, the §6.13 reference-
decomposition table, the §6.18 complex tables, and the §6.19.3 carrier schema. Each
op set is written down two or more times so each section is self-contained; the cost
is that an amendment can add an op to one copy and miss another. A within-doc check
catches that class of typo at the PR, with nothing to run and no implementation to
consult — which is exactly the drift a harness golden-vector test cannot see (the
harness tests bytes, not that ops.md agrees with itself).

The checks (each a set/dict equality between two independent restatements):

  1. primitive floor     §6.3-0001 clause list        == §2.7 primitive-floor table
  2. non-primitive set    §2.7 non-primitive table     == §6.13 decomposition table
  3. non-primitive family §2.7 family column           == §6.13 Family column (per op)
  4. refine set           §6.13 Refine ✓ column        == §6.13-0003 clause list
  5. membership partition  §6.3 / §6.13 / §6.18 op classes are pairwise disjoint (§6.1-0004)
  6. complex op set        §6.18-0001 {...} / §6.18-0016 == §6.18 tables
  7. carrier op set        §2.7 carrier sentence        == §6.19.3 schema table

An OP TOKEN is a backtick span that is ENTIRELY `[a-z][a-z0-9_]{2,}` — every real
KISS op is ≥3 lowercase-alnum chars, so this admits them all while excluding scalar
operands (`a`, `b`, `x`), constants (`const(1)`), and expression bodies (`a+b`,
`|x|`, `e^x`), which is why the false-positive risk is low.

Exit status 0 on agreement, 1 on any drift, so it is a CI gate. Stdlib only.

Usage:
  python tools/kiss_ops.py                  # check
  python tools/kiss_ops.py --emit-coverage  # print the clauses it enforces
"""
from __future__ import annotations

import argparse
import os
import re
import sys

OP_TOKEN = re.compile(r"[a-z][a-z0-9_]{2,}")


def _op(cell):
    """The op token a table cell names, or None: strips backticks/space and returns
    the cell iff it is a single op-shaped token `[a-z][a-z0-9_]{2,}`. This is a shape
    test only — a lowercase family word like `arithmetic` is ALSO op-shaped and would
    pass, so callers apply _op only to op columns (0/2), never to a family column."""
    c = cell.strip().strip("`").strip()
    return c if re.fullmatch(r"[a-z][a-z0-9_]{2,}", c) else None


def op_tokens(text):
    """Every op-shaped token inside its OWN backtick span, in order (dups kept).
    Used where ops are individually backticked and interleaved with prose, so a
    non-backticked prose word is never mistaken for an op."""
    out = []
    for span in re.findall(r"`([^`]*)`", text):
        s = span.strip()
        if re.fullmatch(r"[a-z][a-z0-9_]{2,}", s):
            out.append(s)
    return out


def list_ops(text):
    """Op-shaped tokens from a comma/slash/space-separated LIST, whether or not each
    token is individually backticked — for a set span like `{cmake, cre, cim, ...}`
    (one backtick span of bare words) or a parenthetical `(`a`, `b`, ...)`."""
    out = []
    for w in re.split(r"[,\s/]+", text.replace("`", "").strip()):
        if re.fullmatch(r"[a-z][a-z0-9_]{2,}", w):
            out.append(w)
    return out


class AnchorError(LookupError):
    """A literal anchor this lint navigates by no longer occurs in the spec."""


def between(text, start, end, *, where=""):
    """The slice of `text` strictly between the first occurrence of `start` and the
    next occurrence of `end` after it. `start`/`end` are literal substrings;
    `end=None` runs to EOF deliberately.

    RAISES `AnchorError` when an anchor is absent. It used to return `""` for a
    missing `start` and run to EOF for a missing `end`, and both silently produced
    a WRONG REGION rather than an error:

      * a missing `start` gave an empty region, so the lint checked nothing and
        reported clean — `transcendental_atoms()` returned `[]` for however long
        after §6.8's table was reworded from "Maximum ULP ceiling" to an advisory
        floor;
      * a missing `end` ran to EOF, so `_region_26` in `kiss_dtypes.py` swallowed
        §6.1 and parsed the closed dtype set twice — which a set comparison
        accepts as equal.

    Anchors are prose, and prose gets reworded. **The drift is not preventable;
    the silence is.**

    CHOOSING AN `end` ANCHOR — the trap that has now been hit twice, so it is
    recorded here rather than left for a third person to rediscover. A clause ID
    is NOT a safe end anchor on its own: KISS documents mention a clause ID in
    section-intro prose ABOVE the table the clause governs, so `end="KISS-OPS-
    6.8-0001"` terminates the region BEFORE the table and yields nothing — the
    same empty-region failure, with a brand-new anchor that looks specific.
    Anchor on the BOLDED CLAUSE LINE instead: `"\\n- **KISS-OPS-6.8-0001**"`.
    `sec613_rows()` carries the same warning for §6.13; `transcendental_atoms()`
    was where it bit."""
    i = text.find(start)
    if i < 0:
        raise AnchorError(
            f"anchor not found: start={start!r}{f' [{where}]' if where else ''} — the "
            f"spec text moved. Update the anchor: a region parsed from a missing "
            f"anchor is empty, and an empty region passes every check over it.")
    i += len(start)
    if end is None:
        return text[i:]
    j = text.find(end, i)
    if j < 0:
        raise AnchorError(
            f"anchor not found: end={end!r} after start={start!r}"
            f"{f' [{where}]' if where else ''} — the region would run to EOF and "
            f"swallow every following section.")
    return text[i:j]


def table_cells(region):
    """Yield each markdown table row of `region` as a list of trimmed cell strings.
    Only lines that both start and end with `|` are rows; separator rows (all cells
    empty or dashes) are dropped."""
    for line in region.splitlines():
        s = line.strip()
        if not (s.startswith("|") and s.endswith("|")):
            continue
        cells = [c.strip() for c in s[1:-1].split("|")]
        if all(re.fullmatch(r"-*:?-*", c or "") for c in cells):
            continue
        yield cells


def clause_body(text, clause_short):
    """The body of a `- **KISS-OPS-<short>** — ...` clause, up to the next bolded
    clause bullet or heading. `clause_short` is e.g. `6.3-0001`."""
    m = re.search(
        r"\*\*KISS-OPS-" + re.escape(clause_short) + r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS-OPS|\n#)",
        text, re.S)
    return m.group(1) if m else None


def _diff(where, got, want, out):
    """Append a set-difference violation to `out` if got != want."""
    if set(got) != set(want):
        miss = sorted(set(want) - set(got))
        extra = sorted(set(got) - set(want))
        parts = []
        if miss:
            parts.append(f"missing {miss}")
        if extra:
            parts.append(f"unexpected {extra}")
        out.append(f"{where}: {'; '.join(parts)}")
        return False
    return True


# ---- the seven within-doc restatement sets -----------------------------------

def sec27_primitive(ops):
    return [c[0] for cells in table_cells(
        between(ops, "**Primitive-floor ops", "**Non-primitive ops"))
        if (c := [_op(cells[0])])[0]]


def sec27_nonprimitive(ops):
    """§2.7 non-primitive is a two-op-per-row table (`| op | fam | op | fam |`).
    Take the op token from cells 0 and 2, skipping the trailing empty pairs."""
    ops_out = []
    for cells in table_cells(between(ops, "**Non-primitive ops", "**Complex-arithmetic ops")):
        for i in (0, 2):
            if i < len(cells) and (o := _op(cells[i])):
                ops_out.append(o)
    return ops_out


def sec27_nonprimitive_family(ops):
    """{op -> family} from the §2.7 non-primitive table (op in col 0/2, family in
    the immediately-following col 1/3, plain unbackticked words)."""
    fam = {}
    for cells in table_cells(between(ops, "**Non-primitive ops", "**Complex-arithmetic ops")):
        for i in (0, 2):
            if i + 1 < len(cells) and (o := _op(cells[i])) and cells[i + 1].strip():
                fam[o] = cells[i + 1].strip().strip("`")
    return fam


def sec613_rows(ops):
    """Every data row of the §6.13 decomposition table as (op, family, refine_cell).
    Bounded by the §6.13 heading and the bolded `- **KISS-OPS-6.13-0001**` clause
    line (NOT the earlier prose mention of that ID at ~line 1020)."""
    region = between(ops, "### 6.13 Reference decompositions",
                     "\n- **KISS-OPS-6.13-0001**")
    for cells in table_cells(region):
        if len(cells) >= 3 and (o := _op(cells[0])):
            yield o, cells[1].strip().strip("`"), cells[2]


def sec618_table_ops(ops, which):
    """Op tokens (col 0) of a §6.18 table. which='bridge' -> the component-bridge
    plumbing table; which='advertised' -> the advertised high-level arithmetic table."""
    if which == "bridge":
        region = between(ops, "**Component bridge (plumbing", "**Complex arithmetic (advertised")
    else:
        region = between(ops, "**Complex arithmetic (advertised", "\n- **KISS-OPS-6.18-0001**")
    return [o for cells in table_cells(region) if (o := _op(cells[0]))]


def brace_set(text):
    """The op tokens inside the first `{...}` brace span of `text` (bare or backticked
    words)."""
    m = re.search(r"\{([^{}]*)\}", text)
    return list_ops(m.group(1)) if m else []


def transcendental_atoms(ops):
    """The declared-ULP transcendental atoms — the ops.md §6.8 table. §6.5-0008 cites
    §6.8 for 'every transcendental atom'. Those atoms (exp/log/sin/cos/atan/atan2/erf/
    lgamma/sqrt) live in the PRIMITIVE FLOOR, so they come from the §6.8 table, NOT the
    non-primitive family column (verified against spec/ops.md).

    Anchored on the §6.8 HEADING, not on the table's column header. The previous anchor
    was the column header "Maximum ULP ceiling", which vanished when the table was
    reworded to an *informative advisory floor* — `between()` returned "" and this
    function returned `[]` while its own docstring named nine atoms.

    The end anchor is the BOLDED clause line, not the bare ID: `KISS-OPS-6.8-0001` is
    also mentioned in the section-intro prose ABOVE the table, so ending on the bare ID
    would cut the region before the table and return `[]` again — the same failure with
    a fresh anchor. (`sec613_rows` documents the identical trap.)"""
    region = between(ops, "### 6.8 Transcendental atoms", "\n- **KISS-OPS-6.8-0001**",
                     where="transcendental_atoms")
    atoms = []
    for cells in table_cells(region):
        if cells:
            atoms += op_tokens(cells[0])  # col 0 may list several: `exp`, `log`, ...
    return sorted(set(atoms))


def build_manifest(spec_dir):
    """Derive the op/atom coverage manifest from ops.md (§6.5-0008 checks against it)."""
    ops = open(os.path.join(spec_dir, "ops.md"), encoding="utf-8").read()
    primitive = set(sec27_primitive(ops))
    nonprim = set(sec27_nonprimitive_family(ops))
    all_ops = sorted(primitive | nonprim)
    atoms = transcendental_atoms(ops)  # sqrt, exp, log, sin, cos, atan, atan2, erf, lgamma
    # Plan A's declared coverage set: the exact-byte arithmetic floor that is minted now.
    declared = sorted(o for o in ("add",) if o in all_ops)
    return {
        "schema": "kiss-op-manifest-v1",
        "generated_from": "spec/ops.md",
        "all_ops": all_ops,
        "transcendental_atoms": atoms,
        "declared_coverage_set": declared,
    }


def check(spec_dir):
    ops_path = os.path.join(spec_dir, "ops.md")
    if not os.path.exists(ops_path):
        return [f"missing {ops_path}"]
    ops = open(ops_path, encoding="utf-8").read()
    v = []
    sets = {}

    # (1) primitive floor: §6.3-0001 clause list == §2.7 primitive-floor table
    body = clause_body(ops, "6.3-0001")
    floor_clause = op_tokens(body.split("*Test:*")[0]) if body else []
    floor_27 = sec27_primitive(ops)
    sets["primitive-floor"] = sorted(set(floor_27))
    if not floor_clause:
        v.append("§6.3-0001: no op list found in the clause")
    else:
        _diff("primitive floor (§6.3-0001 clause vs §2.7 table)", floor_clause, floor_27, v)

    # (2) non-primitive set: §2.7 non-primitive table == §6.13 decomposition table
    np_27 = sec27_nonprimitive(ops)
    np_613 = [o for o, _f, _r in sec613_rows(ops)]
    sets["non-primitive"] = sorted(set(np_27))
    _diff("non-primitive set (§2.7 table vs §6.13 table)", np_27, np_613, v)

    # (3) non-primitive family: §2.7 family == §6.13 Family (per op, on the shared keys)
    fam_27 = sec27_nonprimitive_family(ops)
    fam_613 = {o: f for o, f, _r in sec613_rows(ops)}
    for o in sorted(set(fam_27) & set(fam_613)):
        if fam_27[o] != fam_613[o]:
            v.append(f"family drift for `{o}`: §2.7 says '{fam_27[o]}', "
                     f"§6.13 says '{fam_613[o]}'")

    # (4) refine set: §6.13 Refine ✓ column == §6.13-0003 clause list
    refine_table = [o for o, _f, r in sec613_rows(ops) if "✓" in r]
    rbody = clause_body(ops, "6.13-0003")
    # the clause names the refined ops in the parenthetical right after "column ("
    rp = re.search(r"\*Refine\* column[^(]*\(([^)]*)\)", rbody) if rbody else None
    refine_clause = op_tokens(rp.group(1)) if rp else []
    sets["refine"] = sorted(set(refine_table))
    if not refine_clause:
        v.append("§6.13-0003: no refined-op parenthetical list found in the clause")
    else:
        _diff("refine set (§6.13 ✓ column vs §6.13-0003 clause)", refine_table, refine_clause, v)

    # (6) complex op set: §6.18-0001 {...} and the §6.18-0016 split == the §6.18 tables.
    # (The §2.7 complex paragraph is deliberately NOT parsed here: after its op-set
    # sentence it lists real-floor decomposition atoms in backticks, so it is an FP
    # hazard; the machine-clean §6.18-0001/§6.18-0016 clauses are the authoritative copy.)
    body1 = clause_body(ops, "6.18-0001")
    cx_clause = brace_set(body1) if body1 else []
    cx_bridge = sec618_table_ops(ops, "bridge")
    cx_adv = sec618_table_ops(ops, "advertised")
    cx_tables = cx_bridge + cx_adv
    sets["complex"] = sorted(set(cx_clause))
    if not cx_clause:
        v.append("§6.18-0001: no `{...}` complex-op set found in the clause")
    else:
        _diff("complex op set (§6.18-0001 clause vs §6.18 tables)", cx_clause, cx_tables, v)
    # §6.18-0016 partition: advertised == advertised table; bridge == bridge table
    body16 = clause_body(ops, "6.18-0016")
    if body16:
        braces = re.findall(r"\{([^{}]*)\}", body16)
        adv_clause = list_ops(braces[0]) if len(braces) > 0 else []
        bridge_clause = list_ops(braces[1]) if len(braces) > 1 else []
        _diff("advertised-high-level split (§6.18-0016 vs §6.18 advertised table)",
              adv_clause, cx_adv, v)
        _diff("component-bridge split (§6.18-0016 vs §6.18 bridge table)",
              bridge_clause, cx_bridge, v)
    else:
        v.append("§6.18-0016: clause not found")

    # (5) membership partition (§6.1-0004): every op carries exactly one primitive-floor
    # membership flag (primitive = in §6.3; non-primitive = in §6.13 or §6.18), so the
    # three op classes MUST be pairwise disjoint. Without this, the per-class roster
    # checks above could each pass while an op is mis-flagged into two classes at once —
    # which is the specific failure §6.1-0004 forbids.
    classes = [("primitive (§6.3)", set(floor_27)),
               ("non-primitive (§6.13)", set(np_613)),
               ("complex (§6.18)", set(cx_clause))]
    for a in range(len(classes)):
        for b in range(a + 1, len(classes)):
            (an, aset), (bn, bset) = classes[a], classes[b]
            overlap = sorted(aset & bset)
            if overlap:
                v.append(f"membership partition (§6.1-0004): {overlap} in both the "
                         f"{an} and {bn} op classes")

    # (7) carrier set: §2.7 carrier sentence == §6.19.3 schema table (col 0)
    carrier_sentence = between(ops, "The carrier ops for this version are", ". Their schemas")
    carrier_27 = op_tokens(carrier_sentence)
    region = between(ops, "#### 6.19.3", "\n- **KISS-OPS-6.19-0025**")
    carrier_193 = [o for cells in table_cells(region) if (o := _op(cells[0]))]
    sets["carrier"] = sorted(set(carrier_27))
    _diff("carrier set (§2.7 sentence vs §6.19.3 table)", carrier_27, carrier_193, v)

    return v, sets


# The clauses this lint ENFORCES. Only clauses whose drift THIS check would catch and
# that have no harness test are listed (the non-primitive/family/refine/carrier/
# determinism checks strengthen already-backed clauses and add no ledger coverage).
COVERS = [
    ("KISS-OPS-6.3-0001",
     "the primitive-floor op set drifts between the §6.3-0001 clause and the §2.7 table"),
    ("KISS-OPS-6.1-0004",
     "the primitive/non-primitive membership partition drifts between §2.7, §6.3 and §6.13"),
    ("KISS-OPS-6.18-0001",
     "the complex-arithmetic op set drifts between §6.18-0001 and the §6.18 tables"),
    ("KISS-OPS-6.18-0016",
     "the advertised-vs-bridge complex split drifts between §6.18-0016 and the §6.18 tables"),
]


def main():
    ap = argparse.ArgumentParser(description="KISS-Ops within-document op-set consistency lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note)")
    ap.add_argument("--emit-manifest", action="store_true",
                    help="write conformance/corpus/op_manifest.json (the §6.5-0008 coverage source)")
    ap.add_argument("--stdout", action="store_true",
                    help="with --emit-manifest, print the manifest instead of writing the file")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(here), "spec")

    if args.emit_manifest:
        import json as _json
        manifest = build_manifest(spec_dir)
        text = _json.dumps(manifest, indent=2) + "\n"
        if args.stdout:
            # Bytes, not text: `sys.stdout.write` newline-translates on Windows, so
            # the stdout path emitted CRLF while the file path below (which pins the
            # newline explicitly) emitted LF — the SAME generator producing two
            # different artifacts by platform and by output path. A byte-compare gate
            # against either one then fails on the other (#162).
            sys.stdout.buffer.write(text.encode("utf-8"))
        else:
            out_path = os.path.join(os.path.dirname(spec_dir), "conformance", "corpus", "op_manifest.json")
            os.makedirs(os.path.dirname(out_path), exist_ok=True)
            open(out_path, "w", encoding="utf-8", newline="\n").write(text)
            print(f"wrote {out_path}")
        return 0

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    result = check(spec_dir)
    if isinstance(result, list):
        print("KISS-Ops within-doc lint — FATAL")
        for x in result:
            print(f"  - {x}")
        return 1
    violations, sets = result

    print("KISS-Ops within-document op-set consistency lint")
    print("=" * 68)
    for name, s in sets.items():
        print(f"  {name:<16} {len(s)} ops")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} within-doc disagreement(s):")
        for x in violations:
            print(f"      - {x}")
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  every op set ops.md restates agrees with its other copies.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
