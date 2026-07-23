#!/usr/bin/env python3
"""
kiss_vocab.py — the spec <-> reference-binding enumeration lint.

A document lint, complementary to tools/kiss_tables.py. Where kiss_tables checks
that a shared enumeration agrees with ITSELF across the spec (the dtype set, the
determinism enum), this checks that a CLOSED enumeration the spec pins agrees with
the way the reference conformance harness BINDS it — the op-family code set, and
(extensibly) the OpAttrs sub-vocabularies.

Why this matters, and why it is not the same lint as kiss_tables: some closed sets
are restated in several spec files (dtypes, the determinism enum) — kiss_tables
catches a re-fork there. Others are owned by ONE clause and only *referenced* by
tag elsewhere (the 24 op-family codes live only in KISS-Classify §6.5-0006; the
OpAttrs value enums live only in KISS-Ops §6.19 — Grammar/Contract embed OpAttrs
opaquely). For those, a cross-document consistency lint has nothing to disagree
with — it is teethless. The drift that CAN happen is between the pinned set and
the reference implementation that binds it: change the spec's op-family list
without the codec (or vice versa) and the reference reader silently accepts or
rejects the wrong token. This lint is the tooth for exactly that.

It is a reference-BINDING consistency check, not a conformance test: it does not
prove the harness computes a right answer, only that its bound vocabulary matches
the spec it claims to implement. A drift in either direction fails the lint.

Exit status is 0 on agreement, 1 on drift, so it is a CI gate. Stdlib only.

Usage:
  python tools/kiss_vocab.py                  # check
  python tools/kiss_vocab.py --emit-coverage  # print the clauses it enforces
"""
from __future__ import annotations

import argparse
import os
import re
import sys

# A 3-letter lowercase op-family code (KISS-Classify §6.5-0006).
FAMILY_CODE = re.compile(r"`([a-z]{3})`")

NUMBER_WORDS = {
    "twenty-two": 22, "twenty-three": 23, "twenty-four": 24, "twenty-five": 25,
}

# The OpAttrs sub-vocabularies: each is owned by one KISS-Ops §6.19 clause as a
# frozen `{ordinal=name}` enum, restated in the §6.19.2 summary table under a
# field name, and bound in the reference codec as an `ordinal_enum!(Name {...})`.
# (clause_id, rust_enum_name, §6.19.2 table field name).
OPATTRS_ENUMS = [
    ("KISS-OPS-6.19-0014", "Monoid", "monoid"),
    ("KISS-OPS-6.19-0015", "OobPolicy", "oob_policy"),
    ("KISS-OPS-6.19-0016", "ScatterCombine", "scatter_combine"),
    ("KISS-OPS-6.19-0017", "IndexDtype", "index_dtype"),
    ("KISS-OPS-6.19-0018", "SortDirection", "sort_direction"),
    ("KISS-OPS-6.19-0019", "ScanExclusivity", "scan_exclusivity"),
    ("KISS-OPS-6.19-0021", "ColumnOrdering", "column_ordering"),
]

# The closed structure_key sub-vocabularies KISS-Classify §6.5 pins as "MUST be exactly
# {...} with token codes `x`, `y`" and the reference codec binds as a `code_enum!`.
# (clause_id, rust code_enum! name).
CLASSIFY_CODE_ENUMS = [
    ("KISS-CLASSIFY-6.5-0001", "Contig"),      # layout_tag: co/ic/st/br
    ("KISS-CLASSIFY-6.5-0003", "VecWidth"),     # vector width: v1/v2/v4/v8
    ("KISS-CLASSIFY-6.5-0004", "DivBucket"),    # divisibility bucket: d16/d8/d4/d2/da
    ("KISS-CLASSIFY-6.5-0007", "WorkClass"),    # work class: warp/block/grid
    ("KISS-CLASSIFY-6.5-0008", "SizeClass"),    # size class: t/s/m/l
]

# Each entry: a closed set the spec pins and the harness binds, plus the clause a
# drift fails.
COVERS = [("KISS-CLASSIFY-6.5-0006",
           "the 24 op-family codes drift between Classify §6.5-0006 and the "
           "reference structure_key codec (OP_FAMILIES)")]
COVERS += [(cid, f"the {enum} OpAttrs enum drifts between its §6.19 clause, the "
            f"§6.19.2 table, and the reference opattrs codec")
           for cid, enum, _field in OPATTRS_ENUMS]
COVERS += [(cid, f"the {enum} structure_key token-code set drifts between §6.5 and the "
            f"reference code_enum!")
           for cid, enum in CLASSIFY_CODE_ENUMS]
COVERS += [
    ("KISS-CLASSIFY-6.5-0005",
     "the index-width codes ix32/ix64 drift between §6.5-0005 and derive_index_width"),
    ("KISS-CLASSIFY-6.4-0003",
     "the structure_key schema version drifts between §6.4-0003 (sk3) and SCHEMA_VERSION"),
]


def _norm(name):
    """Fold a spec name (`zero-fill`, `atomic-add`, `channel-major (tap-minor)`) and
    a Rust variant (`ZeroFill`, `AtomicAdd`, `ChannelMajor`) to one comparable key:
    lowercase, alnum only, parenthetical dropped."""
    name = name.split("(")[0]
    return re.sub(r"[^a-z0-9]", "", name.lower())


def _markdown_tables(text):
    """Yield each contiguous markdown table as its raw text block."""
    lines, block = text.splitlines(), []
    for line in lines:
        if line.strip().startswith("|") and line.strip().endswith("|"):
            block.append(line)
        elif block:
            yield "\n".join(block)
            block = []
    if block:
        yield "\n".join(block)


def spec_op_families(classify_text):
    """The op-family code set pinned by KISS-Classify §6.5-0006, plus the count word.

    §6.5-0006 says the domain is "exactly the twenty-four categories of the table
    above, each spelled by its 3-letter token code". The table is a two-pair
    layout (`| desc | code | desc | code |`); the op-family table is the one that
    carries the most 3-letter backtick codes.
    """
    best = []
    for block in _markdown_tables(classify_text):
        codes = FAMILY_CODE.findall(block)
        if len(codes) > len(best):
            best = codes
    m = re.search(
        r"op-family-tag domain MUST be exactly the\s+([a-z-]+)\s+categories",
        classify_text)
    count_word = m.group(1) if m else None
    return best, count_word


def harness_op_families(rs_text):
    """The op-family codes bound by the reference codec: the string literals in
    `pub const OP_FAMILIES: [&str; N] = [ "gem", ... ];`."""
    m = re.search(r"OP_FAMILIES\s*:\s*\[&str;\s*\d+\]\s*=\s*\[(.*?)\]", rs_text, re.S)
    if not m:
        return None
    return re.findall(r'"([a-z0-9]+)"', m.group(1))


def spec_ordinal_enum(ops_text, clause_id):
    """The `{ordinal -> normalized name}` map a §6.19 clause pins, e.g. from
    `{1=sum, 2=prod, 3=max, 4=min}`. Returns None if the clause or its brace-form
    is not found."""
    short = clause_id.split("-", 2)[-1]  # e.g. 6.19-0014
    m = re.search(r"\*\*KISS-OPS-" + re.escape(short) + r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)",
                  ops_text, re.S)
    if not m:
        return None
    body = m.group(1)
    brace = re.search(r"\{([^{}]*\d+\s*=[^{}]*)\}", body)
    if not brace:
        return None
    out = {}
    for pair in brace.group(1).split(","):
        pm = re.match(r"\s*(\d+)\s*=\s*(.+)", pair)
        if pm:
            out[int(pm.group(1))] = _norm(pm.group(2))
    return out


def spec_table_enum(ops_text, field):
    """The `{ordinal -> normalized name}` a §6.19.2 sub-vocabulary TABLE row pins,
    e.g. from `| `oob_policy` | enum, `u8` | `0`=RESERVED, `1`=skip, `2`=clamp ... |`.
    `0`=RESERVED is dropped. This is the same enum restated within ops.md, so
    comparing it to the per-clause form is a pure-spec internal-consistency check
    (no harness coupling)."""
    m = re.search(r"^\|\s*`" + re.escape(field) + r"`\s*\|[^|]*\|([^|]*)\|", ops_text, re.M)
    if not m:
        return None
    out = {}
    for pm in re.finditer(r"`?(\d+)`?\s*=\s*([^,|]+)", m.group(1)):
        ordv, name = int(pm.group(1)), _norm(pm.group(2))
        if name != "reserved":
            out[ordv] = name
    return out


def harness_ordinal_enums(opattrs_text):
    """Every reference `ordinal_enum!(Name { Variant = ord, ... })` as
    {enum_name: {ordinal -> normalized variant}}."""
    out = {}
    for m in re.finditer(r"ordinal_enum!\((.*?)\)\s*;", opattrs_text, re.S):
        block = m.group(1)
        nm = re.search(r"(\w+)\s*\{(.*)\}", block, re.S)
        if not nm:
            continue
        name, body = nm.group(1), nm.group(2)
        pairs = {}
        for vm in re.finditer(r"(\w+)\s*=\s*(\d+)", body):
            pairs[int(vm.group(2))] = _norm(vm.group(1))
        out[name] = pairs
    return out


def classify_clause_body(classify_text, clause_id):
    """The body of a `- **KISS-CLASSIFY-<short>** — ...` clause up to the next clause
    bullet or heading."""
    short = clause_id.split("-", 2)[-1]
    m = re.search(r"\*\*KISS-CLASSIFY-" + re.escape(short) +
                  r"\*\*(.*?)(?=\n\s*-\s+\*\*KISS|\n#)", classify_text, re.S)
    return m.group(1) if m else None


def spec_token_codes(classify_text, clause_id):
    """The token-code set a §6.5 clause pins: the FIRST contiguous backtick run after
    the phrase 'token codes' (so a later backtick — e.g. §6.5-0004's `where \\`d16\\`
    selects...` or §6.5-0005's dtype-orthogonality aside — is never swept in)."""
    body = classify_clause_body(classify_text, clause_id)
    if not body:
        return None
    m = re.search(r"token codes\s+((?:`[a-z0-9]+`[\s,/]*)+)", body)
    if not m:
        return None
    return set(re.findall(r"`([a-z0-9]+)`", m.group(1)))


def harness_code_enum(rs_text, name):
    """The code set a reference `code_enum!(Name { Variant = "code", ... })` binds."""
    m = re.search(r"code_enum!\(\s*" + re.escape(name) + r"\s*\{([^}]*)\}", rs_text, re.S)
    if not m:
        return None
    return set(re.findall(r'"([a-z0-9]+)"', m.group(1)))


def harness_index_width_codes(rs_text):
    """The `ixNN` string literals the reference `derive_index_width` returns."""
    m = re.search(r"fn derive_index_width[^{]*\{(.*?)\n\}", rs_text, re.S)
    body = m.group(1) if m else rs_text
    return set(re.findall(r'"(ix\d+)"', body))


def check(spec_dir, conf_dir):
    violations = []
    classify = os.path.join(spec_dir, "classify.md")
    rs = os.path.join(conf_dir, "src", "structure_key.rs")
    if not os.path.exists(classify):
        return [f"missing {classify}"]
    if not os.path.exists(rs):
        return [f"missing {rs} (the reference codec that binds the op-family set)"]

    spec_codes, count_word = spec_op_families(open(classify, encoding="utf-8").read())
    harness_codes = harness_op_families(open(rs, encoding="utf-8").read())
    if harness_codes is None:
        return ["could not find OP_FAMILIES in structure_key.rs"]

    spec_set, harness_set = set(spec_codes), set(harness_codes)

    # the declared count word must match the actual pinned size
    if count_word in NUMBER_WORDS and NUMBER_WORDS[count_word] != len(spec_set):
        violations.append(
            f"§6.5-0006 says '{count_word}' ({NUMBER_WORDS[count_word]}) but the table "
            f"lists {len(spec_set)} distinct codes")
    if len(spec_codes) != len(spec_set):
        dup = sorted({c for c in spec_codes if spec_codes.count(c) > 1})
        violations.append(f"the §6.5 op-family table has duplicate code(s): {dup}")
    if len(harness_codes) != len(harness_set):
        dup = sorted({c for c in harness_codes if harness_codes.count(c) > 1})
        violations.append(f"OP_FAMILIES has duplicate code(s): {dup}")

    if spec_set != harness_set:
        only_spec = sorted(spec_set - harness_set)
        only_harness = sorted(harness_set - spec_set)
        parts = []
        if only_spec:
            parts.append(f"in §6.5-0006 but not the codec: {only_spec}")
        if only_harness:
            parts.append(f"in the codec but not §6.5-0006: {only_harness}")
        violations.append("op-family set drift — " + "; ".join(parts))

    # ---- OpAttrs sub-vocabularies: KISS-Ops §6.19 {ordinal=name} <-> opattrs.rs ----
    ops_md = os.path.join(spec_dir, "ops.md")
    opattrs_rs = os.path.join(conf_dir, "src", "opattrs.rs")
    checked_enums = 0
    if os.path.exists(ops_md) and os.path.exists(opattrs_rs):
        ops_text = open(ops_md, encoding="utf-8").read()
        h_enums = harness_ordinal_enums(open(opattrs_rs, encoding="utf-8").read())
        for clause, enum, field in OPATTRS_ENUMS:
            short = clause.split("-", 2)[-1]
            clause_map = spec_ordinal_enum(ops_text, clause)   # the §6.19-00xx clause
            table_map = spec_table_enum(ops_text, field)       # the §6.19.2 table
            harness_map = h_enums.get(enum)                    # the reference codec
            if clause_map is None:
                violations.append(f"§{short}: no `{{ordinal=name}}` form found in the clause")
                continue
            if table_map is None:
                violations.append(f"§6.19.2 table: no `{field}` row found")
                continue
            if harness_map is None:
                violations.append(f"{enum}: no ordinal_enum! found in opattrs.rs")
                continue
            checked_enums += 1
            # (a) pure-spec internal consistency: the §6.19 clause vs the §6.19.2 table.
            if clause_map != table_map:
                violations.append(
                    f"{field} within-doc drift (§{short} clause vs §6.19.2 table): "
                    f"{dict(sorted(clause_map.items()))} vs {dict(sorted(table_map.items()))}")
            # (b) spec<->binding: the (agreed) spec form vs the reference codec.
            if clause_map != harness_map:
                violations.append(
                    f"{enum} enum drift (§{short} spec vs codec): "
                    f"{dict(sorted(clause_map.items()))} vs {dict(sorted(harness_map.items()))}")
    else:
        violations.append("could not read ops.md or opattrs.rs for the §6.19 enums")

    # ---- Classify §6.5 structure_key sub-vocabularies <-> structure_key.rs codecs ----
    classify_text = open(classify, encoding="utf-8").read()
    rs_text = open(rs, encoding="utf-8").read()
    checked_codes = 0
    for clause, enum_name in CLASSIFY_CODE_ENUMS:
        short = clause.split("-", 2)[-1]
        spec_codes_set = spec_token_codes(classify_text, clause)
        harness_set = harness_code_enum(rs_text, enum_name)
        if spec_codes_set is None:
            violations.append(f"§{short}: no 'token codes `...`' run found in the clause")
            continue
        if harness_set is None:
            violations.append(f"{enum_name}: no code_enum! found in structure_key.rs")
            continue
        checked_codes += 1
        if spec_codes_set != harness_set:
            only_spec = sorted(spec_codes_set - harness_set)
            only_h = sorted(harness_set - spec_codes_set)
            parts = []
            if only_spec:
                parts.append(f"in §{short} but not {enum_name}: {only_spec}")
            if only_h:
                parts.append(f"in {enum_name} but not §{short}: {only_h}")
            violations.append(f"{enum_name} code set drift — " + "; ".join(parts))

    # index-width (§6.5-0005): the `ix32`/`ix64` codes <-> derive_index_width literals
    iw_spec = spec_token_codes(classify_text, "KISS-CLASSIFY-6.5-0005")
    iw_harness = harness_index_width_codes(rs_text)
    if iw_spec and iw_harness:
        checked_codes += 1
        if iw_spec != iw_harness:
            violations.append(f"index-width code drift (§6.5-0005 vs derive_index_width): "
                              f"{sorted(iw_spec)} vs {sorted(iw_harness)}")
    else:
        violations.append("could not extract the §6.5-0005 index-width codes on both sides")

    # schema version (§6.4-0003): the token prefix `sk3` <-> SCHEMA_VERSION
    sv_spec = re.search(r"token prefix `sk(\d+)`", classify_text)
    sv_harness = re.search(r"SCHEMA_VERSION\s*:\s*u32\s*=\s*(\d+)", rs_text)
    if sv_spec and sv_harness:
        checked_codes += 1
        if sv_spec.group(1) != sv_harness.group(1):
            violations.append(f"structure_key schema version drift (§6.4-0003 'sk{sv_spec.group(1)}' "
                              f"vs SCHEMA_VERSION {sv_harness.group(1)})")
    else:
        violations.append("could not extract the structure_key schema version on both sides")

    return violations, (sorted(spec_set), checked_enums, checked_codes)


def main():
    ap = argparse.ArgumentParser(description="KISS spec<->reference-binding enum lint")
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--conformance-dir", default=None)
    ap.add_argument("--emit-coverage", action="store_true",
                    help="print the clause IDs this lint enforces (clause<TAB>note)")
    args = ap.parse_args()
    here = os.path.dirname(os.path.abspath(__file__))
    root = os.path.dirname(here)
    spec_dir = args.spec_dir or os.path.join(root, "spec")
    conf_dir = args.conformance_dir or os.path.join(root, "conformance")

    if args.emit_coverage:
        for cid, note in COVERS:
            print(f"{cid}\t{note}")
        return 0

    result = check(spec_dir, conf_dir)
    if isinstance(result, list):
        print("KISS vocab lint — FATAL")
        for v in result:
            print(f"  - {v}")
        return 1
    violations, (codes, n_enums, n_codes) = result

    print("KISS spec<->binding enumeration lint")
    print("=" * 68)
    print(f"  op-family codes (KISS-CLASSIFY-6.5-0006): {len(codes)}")
    print(f"    {' '.join(codes)}")
    print(f"  OpAttrs enums checked (KISS-Ops §6.19): {n_enums}  "
          f"(§6.19 clause <-> §6.19.2 table <-> codec)")
    print(f"  structure_key sub-vocabularies checked (KISS-Classify §6.5): {n_codes}  "
          f"(§6.5 clause <-> code_enum!/derive/SCHEMA_VERSION)")
    print("-" * 68)
    if violations:
        print(f"  DRIFT — {len(violations)} disagreement(s):")
        for v in violations:
            print(f"      - {v}")
        print("  RESULT: DRIFT FOUND")
        return 1
    print("  op-family, the 7 OpAttrs enums, and the §6.5 structure_key sub-vocabularies")
    print("  all agree with the reference codec.")
    print("  RESULT: CLEAN")
    return 0


if __name__ == "__main__":
    sys.exit(main())
