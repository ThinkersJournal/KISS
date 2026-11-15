#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""
kiss_op_facts.py — the machine-readable per-op structural facts, GENERATED from spec/ops.md.

`conformance/corpus/op_facts.json` carries, per op, the structural facts a consumer needs to
classify a cell (KISS-CLASSIFY §6.3) or build an op DAG (§6.4) without hand-copying prose:

  family               the §2.7 / §6.13 family word (`contraction`, `reduction`, ...)
  carrier              the op carries a non-empty OpAttrs record (§6.19-0003)
  opattrs              the carrier's OpAttrs field names in canonical (wire) order (§6.19.3)
  reduce_axes          the OpAttrs axis-set field the op folds over: `reduce_axes`, `norm_axes`,
                       or null (§6.11-0011, §6.19-0020, §6.19-0031)
  is_contraction       family `contraction` (the §6.13 table; §6.13-0009's `matmul` contraction)
  structured_op        a §6.13-0009 structured op (decomposes onto a named §6.11 structural op)
  index_operand_role   carries the explicit index-operand role (§6.11-0012)

Nothing here is authored: every value is read from a table or clause in ops.md, and the
generator CROSS-CHECKS the restatements it reads (a table and the clause that lists the same set
must agree) so a reworded anchor fails loudly instead of emitting an empty fact. The committed file
is gated by CI exactly as op_manifest.json is: regenerate, `git diff --exit-code`.

Usage:
  python tools/kiss_op_facts.py                 # write conformance/corpus/op_facts.json
  python tools/kiss_op_facts.py --stdout        # print it (bytes, LF)
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import kiss_ops as K  # noqa: E402

SCHEMA = "kiss-op-facts-v1"
# The clause each fact is read from, so a consumer can cite it (never re-derive it).
FACT_CLAUSES = {
    "family": ["KISS-OPS-6.1-0001", "KISS-OPS-6.13-0001"],
    "carrier": ["KISS-OPS-6.19-0003"],
    "opattrs": ["KISS-OPS-6.19-0004"],
    "reduce_axes": ["KISS-OPS-6.11-0011", "KISS-OPS-6.19-0020", "KISS-OPS-6.19-0031"],
    "is_contraction": ["KISS-OPS-6.13-0009"],
    "structured_op": ["KISS-OPS-6.13-0009"],
    "index_operand_role": ["KISS-OPS-6.11-0012"],
}
AXIS_FIELDS = ("reduce_axes", "norm_axes")
FIELD_NAME = re.compile(r"(?:^|— )`([a-z_0-9]+)`:")  # a field starts the cell or follows " — "; `u8`: is an encoding


def _families(ops):
    """{op -> family} over the §2.7 primitive table plus the §2.7 non-primitive families
    (the §6.13 table's own Family column is cross-checked against the latter by kiss_ops)."""
    fam = dict(K.sec27_nonprimitive_family(ops))
    region = K.between(ops, "**Primitive-floor ops", "**Non-primitive ops", where="§2.7 primitive")
    for cells in K.table_cells(region):
        if len(cells) >= 2 and (o := K._op(cells[0])):
            fam[o] = cells[1].strip().strip("`")
    for o in K.sec618_table_ops(ops, "bridge"):
        fam[o] = "complex-bridge"  # §6.18 component-bridge plumbing table has no Family column
    region = K.between(ops, "**Complex arithmetic (advertised", "\n- **KISS-OPS-6.18-0001**", where="§6.18")
    for cells in K.table_cells(region):
        if len(cells) >= 2 and (o := K._op(cells[0])):
            fam[o] = cells[1].strip().strip("`")
    return fam


def _opattrs(ops):
    """{op -> [field names in canonical order]} from the §6.19.3 schema table."""
    region = K.between(ops, "#### 6.19.3 Per-op OpAttrs schema", "\n#### ", where="§6.19.3")
    out = {}
    for cells in K.table_cells(region):
        if len(cells) >= 2 and (o := K._op(cells[0])):
            out[o] = FIELD_NAME.findall(cells[1])
    return out


def _clause_ops(ops, short, start, end):
    """Op tokens in the backticked list a clause body states between two literal anchors."""
    body = K.clause_body(ops, short)
    if body is None:
        raise K.AnchorError(f"clause KISS-OPS-{short} not found")
    return K.op_tokens(K.between(body, start, end, where=f"KISS-OPS-{short}"))


def _carrier_set(ops):
    body = K.clause_body(ops, "6.19-0003")
    if body is None:
        raise K.AnchorError("clause KISS-OPS-6.19-0003 not found")
    return set(K.brace_set(body))


def build_op_facts(spec_dir):
    ops = open(os.path.join(spec_dir, "ops.md"), encoding="utf-8").read().replace("\r\n", "\n")
    manifest = K.build_manifest(spec_dir)
    fam, attrs = _families(ops), _opattrs(ops)
    structured = set(_clause_ops(ops, "6.13-0009", "structured op** —", "— MUST"))
    index_role = set(_clause_ops(ops, "6.11-0012", "", "MUST carry the index-operand role"))
    carriers = _carrier_set(ops)

    problems = []
    K._diff("carrier set (§6.19-0003) vs §6.19.3 schema table", attrs, carriers, problems)
    for name, want in (("structured_op set", structured), ("index_operand_role set", index_role)):
        if not want:
            problems.append(f"{name}: empty — its clause anchor moved")
        for o in sorted(want - set(manifest["all_ops"])):
            problems.append(f"{name}: {o!r} is not an op in the manifest")
    for o in manifest["all_ops"]:
        if o not in fam:
            problems.append(f"family: no family for op {o!r}")
    if problems:
        raise SystemExit("kiss_op_facts: spec/ops.md restatements disagree:\n  - " + "\n  - ".join(problems))

    facts = {}
    for o in manifest["all_ops"]:
        fields = attrs.get(o, [])
        axis = [f for f in fields if f in AXIS_FIELDS]
        facts[o] = {
            "family": fam[o],
            "carrier": o in carriers,
            "opattrs": fields,
            "reduce_axes": axis[0] if axis else None,
            "is_contraction": fam[o] == "contraction",
            "structured_op": o in structured,
            "index_operand_role": o in index_role,
        }
    return {
        "schema": SCHEMA,
        "generated_from": "tools/kiss_op_facts.py (python tools/kiss_op_facts.py; reads spec/ops.md)",
        "fact_clauses": FACT_CLAUSES,
        "ops": facts,
    }


def render(spec_dir):
    return (json.dumps(build_op_facts(spec_dir), indent=2) + "\n").encode("utf-8")


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--spec-dir", default=None)
    ap.add_argument("--stdout", action="store_true")
    args = ap.parse_args()
    spec_dir = args.spec_dir or os.path.join(os.path.dirname(HERE), "spec")
    data = render(spec_dir)
    if args.stdout:
        sys.stdout.buffer.write(data)
        return 0
    out = os.path.join(os.path.dirname(spec_dir), "conformance", "corpus", "op_facts.json")
    with open(out, "wb") as f:
        f.write(data)
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
