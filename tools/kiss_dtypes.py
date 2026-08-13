#!/usr/bin/env python3
"""
kiss_dtypes.py — the KISS-Classify §6.1 dtype-set SSOT manifest + within-doc lint.

A sibling of tools/kiss_ops.py. KISS-Classify §6.1 pins the scalar dtype set as a
closed normative table, and RESTATES it as the §2.6 readable catalog and again inside
the §6.1-0001 clause token list. Four token-deriving projects each transcribe that
22-row table by hand into their vocabulary crates, which is a drift generator — one
party silently shipped 17 of the 22 (dropping s16/u16/u64/e4m3fnuz/e5m2fnuz, and
parsing the two reserved `fnuz` tokens as *unknown*, which §6.1-0001 forbids). A
machine-readable manifest lets a party GENERATE its dtype vocabulary from KISS rather
than copy it: a generated 22-row enum cannot be missing a row.

This tool:

  1. --emit-manifest  writes conformance/corpus/dtype_manifest.json — the SSOT the
     token-deriving parties generate from (per row: token, kind, storage_bits,
     reserved). KISS owns §6.1 outright, so KISS is the unambiguous source (no
     §6.8-0004 maintainer-ownership question, unlike a namespace vocabulary).

  2. default (no args) is a within-document lint: the §6.1 normative table, the §2.6
     readable catalog, and the §6.1-0001 clause token list MUST name the same 22
     tokens with the same kind and width. Exit 1 on drift (a CI gate), so an amendment
     that edits one copy and misses another fails at the PR — the same self-contained,
     nothing-to-run discipline as kiss_ops.py.

Scope: the manifest carries the CLOSED SET + storage metadata ONLY — never a party's
Rust variant names, C type spellings, or which dtypes a backend can lower, all of
which are legitimately local. Generating the vocabulary fixes the axis where drift
happens; lowering coverage stays per-implementation.

Stdlib only; no harness coupling.

Usage:
  python tools/kiss_dtypes.py                    # within-doc lint (CI gate)
  python tools/kiss_dtypes.py --emit-manifest    # write dtype_manifest.json
  python tools/kiss_dtypes.py --emit-manifest --stdout
"""
from __future__ import annotations

import argparse
import os
import re
import sys

# Dtype tokens are lowercase alphanumeric, >=2 chars, with NO underscore: f16, bf16,
# s8, u8, b1, s4, c32, e4m3fnuz. (kiss_ops.py's op-token regex requires >=3 chars,
# which would wrongly exclude the 2-char dtypes s8/u8/s4/u4/b1 — dtypes need their
# own shape.)
DTYPE_TOKEN = re.compile(r"[a-z][a-z0-9]+")


class AnchorError(LookupError):
    """A literal anchor this lint navigates by no longer occurs in the spec."""


def between(text, start, end, *, where=""):
    """The slice strictly between the first `start` and the next `end` after it;
    `end=None` runs to EOF deliberately.

    RAISES `AnchorError` on an absent anchor. The silent forms — `""` for a missing
    start, run-to-EOF for a missing end — both returned a WRONG REGION rather than
    an error, and both have already bitten this suite: `_region_26` below records
    the run-to-EOF instance (it swallowed §6.1 and parsed the closed dtype set
    twice, which a set comparison accepts as equal), and `kiss_ops.py`'s
    `transcendental_atoms()` records the empty-region one (it returned `[]` for
    however long after §6.8 was reworded).

    Kept deliberately identical to `kiss_ops.between` — same defect, same fix, and
    a divergence between the two would be its own Pattern A."""
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


def table_rows(region):
    """Yield each markdown table row of `region` as trimmed cells; drop separator
    rows (all dashes)."""
    for line in region.splitlines():
        s = line.strip()
        if not (s.startswith("|") and s.endswith("|")):
            continue
        cells = [c.strip() for c in s[1:-1].split("|")]
        if all(re.fullmatch(r"-*:?-*", c or "") for c in cells):
            continue
        yield cells


def _token(cell):
    """The dtype token a cell names, or None: strips backticks/space and returns it
    iff it is exactly one dtype-shaped token. The header cell 'Token' is capitalized,
    so it is rejected here — no explicit header skip needed."""
    c = cell.strip().strip("`").strip()
    return c if re.fullmatch(DTYPE_TOKEN, c) else None


def parse_table(region):
    """Parse a dtype table region (columns: Token, Kind, bits, Notes) into a list of
    {token, kind, storage_bits, reserved}, in table order. Non-dtype rows (header,
    prose) are skipped."""
    rows = []
    for cells in table_rows(region):
        if len(cells) < 3:
            continue
        tok = _token(cells[0])
        if not tok:
            continue
        try:
            bits = int(cells[2].strip())
        except ValueError:
            continue
        note = cells[3] if len(cells) > 3 else ""
        rows.append({
            "token": tok,
            "kind": cells[1].strip().strip("`"),
            "storage_bits": bits,
            "reserved": "reserved" in note.lower(),
        })
    return rows


def clause_tokens(text):
    """The dtype tokens in the §6.1-0001 parenthetical
    ('...twenty-<N> tokens in the table above ( `f16`, `bf16`, ... )'). Version-agnostic:
    anchors on 'tokens in the table above (' so the count word (twenty-two at sk3,
    twenty-four at sk4) is not baked in."""
    body = between(text, "tokens in the table above (", ")")
    return [t for span in re.findall(r"`([^`]*)`", body)
            if (t := span.strip()) and re.fullmatch(DTYPE_TOKEN, t)]


def _region_61(text):
    return between(text, "### 6.1 The pinned scalar dtype set",
                  "\n- **KISS-CLASSIFY-6.1-0001**")


def _region_26(text):
    # Terminate on the NEXT section header (§2.7), not the count word: the count is
    # version-specific ("Twenty-two" at sk3, "Twenty-four" at sk4), and baking it in
    # made between() run to EOF once the word changed at sk4 — swallowing §6.1 and
    # parsing the closed set TWICE, which a set comparison accepts as equal (the
    # §2.6-omits-a-dtype drift then hides behind §6.1's copy). Version-agnostic, the
    # way clause_tokens() already is. The row-count / duplicate guard in check() is the
    # loud backstop for any future terminator break.
    return between(text, "### 2.6 Readable catalog", "\n### 2.7")


def schema_version(text):
    """The structure_key schema version and token prefix, from KISS-CLASSIFY-6.4-0003
    ('...token prefix `sk3`', §6.7-0002). Clause D (§3.4) forbids persisting, indexing,
    or comparing dtype sub-tokens detached from this version; a manifest IS a persisted,
    indexed list of them, so it MUST carry the version to be self-describing under the
    same rule it distributes. Version and prefix are one axis (`sk<N>`), parsed from the
    single clause that pins them. Anchored to the clause BULLET and scoped to the clause
    body (not a first-substring match + fixed window, which a TOC or cross-reference
    could shadow)."""
    m = re.search(
        r"-\s+\*\*KISS-CLASSIFY-6\.4-0003\*\*(.*?)(?=\n\s*-\s+\*\*KISS-CLASSIFY|\n#)",
        text, re.S)
    if not m:
        raise ValueError("KISS-CLASSIFY-6.4-0003 clause bullet not found")
    mv = re.search(r"`sk(\d+)`", m.group(1))
    if not mv:
        raise ValueError("structure_key token prefix `sk<N>` not found in KISS-CLASSIFY-6.4-0003")
    return int(mv.group(1)), "sk" + mv.group(1)


def build_manifest(spec_dir):
    """Derive the dtype manifest from the §6.1 normative table (the authoritative
    copy)."""
    text = open(os.path.join(spec_dir, "classify.md"), encoding="utf-8").read()
    dtypes = parse_table(_region_61(text))
    version, prefix = schema_version(text)
    return {
        "schema": "kiss-dtype-manifest-v1",
        "generated_from": "spec/classify.md",
        "clause": "KISS-CLASSIFY-6.1-0001",
        # Clause D (§3.4): a dtype sub-token has no meaning detached from the
        # structure_key schema version that produced it, and MUST NOT be persisted or
        # indexed independently of it. These two fields make this persisted list
        # self-describing — and they flip in the same diff as the spellings at the sk4
        # cut (e.g. c64 goes pair-of-f64 -> pair-of-f32), so a vendored copy always
        # knows which vocabulary it pinned.
        "structure_key_schema_version": version,
        "token_prefix": prefix,
        "dtypes": dtypes,
        "all_dtypes": sorted(d["token"] for d in dtypes),
        "kinds": sorted({d["kind"] for d in dtypes}),
    }


def check(spec_dir):
    path = os.path.join(spec_dir, "classify.md")
    if not os.path.exists(path):
        return [f"missing {path}"]
    text = open(path, encoding="utf-8").read()
    v = []
    t61 = parse_table(_region_61(text))
    t26 = parse_table(_region_26(text))
    cl = clause_tokens(text)

    s61 = {d["token"] for d in t61}
    s26 = {d["token"] for d in t26}
    scl = set(cl)

    if not t61:
        v.append("§6.1 normative dtype table not found or empty")
    if not t26:
        v.append("§2.6 readable catalog not found or empty")
    if not cl:
        v.append("§6.1-0001 clause token list not found or empty")

    def diff(where, got, want):
        if got != want:
            miss = sorted(want - got)
            extra = sorted(got - want)
            parts = []
            if miss:
                parts.append(f"missing {miss}")
            if extra:
                parts.append(f"unexpected {extra}")
            v.append(f"{where}: {'; '.join(parts)}")

    # the three copies of the closed set must name the same tokens
    diff("§2.6 readable catalog vs §6.1 normative table", s26, s61)
    diff("§6.1-0001 clause list vs §6.1 normative table", scl, s61)

    # ROW-COUNT / DUPLICATE guard (version-agnostic). A set comparison alone is
    # vacuous against a doubled region: if _region_26 loses its terminator and runs to
    # EOF it swallows §6.1, so §2.6 parses the closed set TWICE — 48 rows whose SET is
    # still the 24 tokens, so the diff above passes while the §2.6-omits-a-dtype drift
    # (the very drift this lint exists to catch) hides behind §6.1's copy. Guard the
    # COUNT (the two tables must be equal length) and reject duplicate tokens in either
    # table (the direct signature of a doubled region). No absolute count is baked in.
    if t26 and t61 and len(t26) != len(t61):
        v.append(f"§2.6 catalog has {len(t26)} rows but §6.1 table has {len(t61)} — "
                 f"row-count mismatch (a broken §2.6 region terminator doubles it by "
                 f"swallowing §6.1)")
    for where, rows in (("§2.6 readable catalog", t26), ("§6.1 normative table", t61)):
        toks = [d["token"] for d in rows]
        dups = sorted({t for t in toks if toks.count(t) > 1})
        if dups:
            v.append(f"{where}: duplicate dtype rows {dups} (a doubled region or a "
                     f"copy-paste — a set comparison cannot see this)")

    # §6.1-0001 pins the closed set; the three-way clause-vs-table-vs-catalog equality
    # above is the guard. No hardcoded absolute count — it is version-specific (22 at
    # sk3, 24 at sk4) and would need editing every schema bump, and the set-agreement is
    # what actually enforces "exactly the N tokens listed".

    # kind and width must agree between the two tables, per shared token
    m26 = {d["token"]: d for d in t26}
    for d in t61:
        o = m26.get(d["token"])
        if not o:
            continue
        if o["kind"] != d["kind"]:
            v.append(f"kind drift for `{d['token']}`: §2.6 '{o['kind']}' vs §6.1 '{d['kind']}'")
        if o["storage_bits"] != d["storage_bits"]:
            v.append(f"width drift for `{d['token']}`: §2.6 {o['storage_bits']} vs §6.1 {d['storage_bits']}")

    return v


# The clause this lint guards. §6.1-0001 already has a harness test
# (test_classify_dtype_set_is_closed), so the within-doc lint STRENGTHENS it and adds
# no ledger coverage; the manifest is the new SSOT artifact, the deliverable.
COVERS = [
    ("KISS-CLASSIFY-6.1-0001",
     "the closed dtype set / kind / width drifts between the §6.1 table, the §2.6 "
     "readable catalog, and the §6.1-0001 clause list"),
]


def main():
    ap = argparse.ArgumentParser(description="KISS-Classify §6.1 dtype manifest + within-doc lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--emit-manifest", action="store_true",
                    help="write conformance/corpus/dtype_manifest.json (the §6.1 SSOT)")
    ap.add_argument("--stdout", action="store_true",
                    help="with --emit-manifest, print instead of writing the file")
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint guards (clause<TAB>note)")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(here), "spec")

    if args.emit_manifest:
        import json as _json
        text = _json.dumps(build_manifest(spec_dir), indent=2) + "\n"
        if args.stdout:
            sys.stdout.write(text)
        else:
            out = os.path.join(os.path.dirname(spec_dir), "conformance", "corpus", "dtype_manifest.json")
            os.makedirs(os.path.dirname(out), exist_ok=True)
            open(out, "w", encoding="utf-8", newline="\n").write(text)
            print(f"wrote {out}")
        return 0

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    violations = check(spec_dir)
    print("KISS-Classify §6.1 dtype within-document consistency lint")
    print("=" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} disagreement(s):")
        for x in violations:
            print(f"      - {x}")
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  §6.1 table, §2.6 catalog, and §6.1-0001 clause list agree.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
